#!/usr/bin/env bash
# Hardens the live production host (GUIDE section 8) in place: SSH by key only,
# ufw with the recorded ports, unattended security upgrades, Docker from its own
# apt repository, swap, time sync, and a deploy user without sudo. Every run
# converges and then verifies; a second run changes nothing. Rules, services, and
# files it does not manage (ufw-docker in after.rules, the OCI rules in
# before.rules, other ufw rules) are left alone.
#
# Run as root through sudo from the admin account (`make prod-provision HOST=`):
#   sudo bash provision_host.sh           converge, then verify
#   sudo bash provision_host.sh --check   verify only, change nothing
# Environment:
#   DEPLOY_PUBKEY  one public key for the deploy user, installed behind the forced command
#   BESZEL_HUB     address of the Beszel hub (default 10.0.0.100)
#   REBOOT_TIME    unattended-upgrades reboot time, host clock (default 19:00, 02:00 WIB on UTC)
#   SWAP_SIZE      swapfile size when the host has no swap (default 4G)
# shellcheck disable=SC2015 # ok() always succeeds, so `test && ok || bad` is an if-else
set -euo pipefail

BESZEL_HUB=${BESZEL_HUB:-10.0.0.100}
BESZEL_PORT=45876
REBOOT_TIME=${REBOOT_TIME:-19:00}
SWAP_SIZE=${SWAP_SIZE:-4G}
DEPLOY_PUBKEY=${DEPLOY_PUBKEY:-}
DEPLOY_USER=deploy
DEPLOY_COMMAND=/usr/local/bin/rotaria-deploy
APP_DIR=/srv/rotaria
ENV_FILE=$APP_DIR/repo/.env.production
SSHD_DROPIN=/etc/ssh/sshd_config.d/00-rotaria.conf
APT_DROPIN=/etc/apt/apt.conf.d/52rotaria-unattended-upgrades
DOCKER_KEY_FINGERPRINT=9DC858229FC7DD38854AE2D88D81803C0EBFCD88
# Inbound ports the single-host design no longer opens (ADR-46): web edge and data stores.
OBSOLETE_PORTS=(80 443 5432 6379)

CHANGES=()
FAILURES=()

log() { printf '%s\n' "$*"; }
die() { printf 'provision: %s\n' "$*" >&2; exit 2; }
changed() { CHANGES+=("$1"); log "changed: $1"; }
ok() { log "ok      $1"; }
bad() { FAILURES+=("$1"); log "FAIL    $1"; }

# Writes a file only when its content, mode, or owner differ.
put_file() {
    local path=$1 mode=$2 owner=$3 content=$4 tmp
    tmp=$(mktemp)
    printf '%s\n' "$content" >"$tmp"
    if [[ -f $path ]] && cmp -s "$tmp" "$path" \
        && [[ $(stat -c '%a %U:%G' "$path") == "$mode $owner" ]]; then
        rm -f "$tmp"
        return 1
    fi
    install -m "$mode" -o "${owner%%:*}" -g "${owner##*:}" "$tmp" "$path"
    rm -f "$tmp"
    changed "$path"
}

is_installed() {
    [[ $(dpkg-query -W -f='${db:Status-Status}' "$1" 2>/dev/null) == installed ]]
}

APT_UPDATED=0
ensure_packages() {
    local missing=() pkg
    for pkg in "$@"; do is_installed "$pkg" || missing+=("$pkg"); done
    ((${#missing[@]})) || return 0
    if ((!APT_UPDATED)); then apt-get update -q; APT_UPDATED=1; fi
    apt-get install -y -q --no-install-recommends -o Dpkg::Options::=--force-confold "${missing[@]}"
    changed "packages ${missing[*]}"
}

ufw_do() {
    local out
    out=$(ufw "$@")
    if grep -qE '^Rules? (added|updated|deleted)' <<<"$out"; then changed "ufw $*"; fi
}

preflight() {
    ((EUID == 0)) || die "run as root through sudo from the admin account"
    # shellcheck source=/dev/null
    . /etc/os-release
    [[ ${ID:-} == ubuntu ]] || die "expects Ubuntu, found ${ID:-unknown}"
    # Root login and passwords are about to go; the account running this must keep key access.
    local admin=${SUDO_USER:-} home
    [[ -n $admin && $admin != root ]] || die "run through sudo from the admin account, not as root, so its key access can be confirmed"
    home=$(getent passwd "$admin" | cut -d: -f6)
    grep -qE '^[^#[:space:]]' "$home/.ssh/authorized_keys" 2>/dev/null \
        || die "$admin has no key in $home/.ssh/authorized_keys; add one before SSH turns key-only"
    ADMIN_USER=$admin
    # sshd -T needs its privilege separation directory, which socket activation creates only on demand.
    install -d -m 0755 /run/sshd
    # The firewall only opens 22; an sshd on another port would be cut off when ufw is enabled.
    grep -qx 'port 22' <<<"$(sshd -T 2>/dev/null)" || die "sshd does not listen on port 22; adjust the ufw rules first"
}

ensure_firewall() {
    ensure_packages ufw
    # SSH and Beszel go in before anything is removed or the firewall is enabled.
    ufw_do allow 22/tcp
    ufw_do allow from "$BESZEL_HUB" to any port "$BESZEL_PORT" proto tcp
    remove_obsolete_rules
    if ! grep -q '^DEFAULT_INPUT_POLICY="DROP"' /etc/default/ufw; then
        ufw default deny incoming >/dev/null
        changed "ufw default deny incoming"
    fi
    if ! grep -q '^DEFAULT_OUTPUT_POLICY="ACCEPT"' /etc/default/ufw; then
        ufw default allow outgoing >/dev/null
        changed "ufw default allow outgoing"
    fi
    if grep -q '^Status: inactive' <<<"$(ufw status)"; then
        ufw --force enable >/dev/null
        changed "ufw enabled"
    fi
}

# Prints the ports an `ufw show added` rule names, comma separated, or nothing.
rule_ports() {
    local rule=$1 last=${1##* }
    if [[ $rule =~ port\ ([^ ]+) ]]; then
        printf '%s' "${BASH_REMATCH[1]}"
    elif [[ $last =~ ^[0-9][0-9,:]*(/[a-z]+)?$ ]]; then
        printf '%s' "${last%%/*}"
    fi
}

is_obsolete_port() {
    local p
    for p in "${OBSOLETE_PORTS[@]}"; do [[ $1 == "$p" ]] && return 0; done
    return 1
}

remove_obsolete_rules() {
    local rule ports port hit keep
    local -a list
    while IFS= read -r rule; do
        [[ $rule =~ ^ufw\ (route\ )?(allow|limit)\  ]] || continue
        ports=$(rule_ports "$rule")
        [[ -n $ports ]] || continue
        IFS=, read -ra list <<<"$ports"
        hit=0 keep=0
        for port in "${list[@]}"; do
            if is_obsolete_port "$port"; then hit=1; else keep=1; fi
        done
        ((hit)) || continue
        # A rule that also opens a port still in use (say 22,80/tcp) is left for a person.
        if ((keep)); then
            log "skip    '$rule' mixes kept and obsolete ports; split it by hand"
            continue
        fi
        # shellcheck disable=SC2086 # the rule is ufw's own word list
        if [[ $rule == "ufw route "* ]]; then
            ufw_do route delete ${rule#ufw route }
        else
            ufw_do delete ${rule#ufw }
        fi
    done < <(ufw show added | grep '^ufw ')
}

ensure_sshd() {
    ensure_packages openssh-server
    local previous=""
    [[ -f $SSHD_DROPIN ]] && previous=$(cat "$SSHD_DROPIN")
    # sshd keeps the first value it reads and the stock config includes sshd_config.d
    # first, so this file sorts ahead of 50-cloud-init.conf and the main file.
    put_file "$SSHD_DROPIN" 644 root:root "# Managed by scripts/provision_host.sh (GUIDE section 8).
PermitRootLogin no
PasswordAuthentication no
KbdInteractiveAuthentication no
PermitEmptyPasswords no
PubkeyAuthentication yes
AuthenticationMethods publickey
X11Forwarding no
LoginGraceTime 30" || return 0
    if ! sshd -t; then
        if [[ -n $previous ]]; then printf '%s\n' "$previous" >"$SSHD_DROPIN"; else rm -f "$SSHD_DROPIN"; fi
        die "sshd rejected the new config; restored the previous $SSHD_DROPIN"
    fi
    # A reload keeps open sessions; with socket activation an idle sshd reads the file on the next connection.
    systemctl try-reload-or-restart ssh.service
}

ensure_deploy_user() {
    if ! id "$DEPLOY_USER" >/dev/null 2>&1; then
        useradd --create-home --user-group --shell /bin/bash "$DEPLOY_USER"
        changed "user $DEPLOY_USER"
    fi
    local group home
    for group in sudo admin wheel; do
        if [[ " $(id -nG "$DEPLOY_USER") " == *" $group "* ]]; then
            gpasswd -d "$DEPLOY_USER" "$group" >/dev/null
            changed "$DEPLOY_USER removed from $group"
        fi
    done
    if [[ $(passwd -S "$DEPLOY_USER" | cut -d' ' -f2) != L ]]; then
        usermod -L "$DEPLOY_USER"
        changed "$DEPLOY_USER password locked"
    fi
    # Placeholder until the deploy command exists; never overwrites the real one.
    if [[ ! -e $DEPLOY_COMMAND ]]; then
        put_file "$DEPLOY_COMMAND" 755 root:root '#!/bin/sh
# Placeholder forced command for the deploy key. The real deploy command replaces this file.
echo "rotaria-deploy: placeholder, no deploy command is installed on this host yet" >&2
exit 1' || true
    fi
    home=$(getent passwd "$DEPLOY_USER" | cut -d: -f6)
    if [[ ! -d $home/.ssh ]]; then
        install -d -m 0700 -o "$DEPLOY_USER" -g "$DEPLOY_USER" "$home/.ssh"
        changed "$home/.ssh"
    fi
    [[ -n $DEPLOY_PUBKEY ]] || return 0
    [[ $DEPLOY_PUBKEY != *$'\n'* && $DEPLOY_PUBKEY =~ ^(ssh-|ecdsa-|sk-) ]] \
        || die "DEPLOY_PUBKEY must be one public key line without options"
    ssh-keygen -l -f /dev/stdin <<<"$DEPLOY_PUBKEY" >/dev/null 2>&1 \
        || die "DEPLOY_PUBKEY is not a valid public key"
    put_file "$home/.ssh/authorized_keys" 600 "$DEPLOY_USER:$DEPLOY_USER" \
        "restrict,command=\"$DEPLOY_COMMAND\" $DEPLOY_PUBKEY" || true
}

ensure_app_dir() {
    if [[ ! -d $APP_DIR ]]; then
        install -d -m 0755 "$APP_DIR"
        changed "$APP_DIR"
    fi
    if [[ -f $ENV_FILE && $(stat -c '%a' "$ENV_FILE") != 600 ]]; then
        chmod 600 "$ENV_FILE"
        changed "$ENV_FILE mode 600"
    fi
}

ensure_unattended_upgrades() {
    ensure_packages unattended-upgrades
    # Sorts after 50unattended-upgrades and 20auto-upgrades, so these values win.
    put_file "$APT_DROPIN" 644 root:root "// Managed by scripts/provision_host.sh (GUIDE section 8).
APT::Periodic::Update-Package-Lists \"1\";
APT::Periodic::Unattended-Upgrade \"1\";
Unattended-Upgrade::Automatic-Reboot \"true\";
Unattended-Upgrade::Automatic-Reboot-WithUsers \"true\";
Unattended-Upgrade::Automatic-Reboot-Time \"$REBOOT_TIME\";
Unattended-Upgrade::Remove-Unused-Kernel-Packages \"true\";" || true
    local unit
    for unit in apt-daily.timer apt-daily-upgrade.timer; do
        if ! systemctl is-enabled --quiet "$unit"; then
            systemctl enable --now "$unit"
            changed "$unit enabled"
        fi
    done
}

time_unit() {
    local unit
    for unit in chrony systemd-timesyncd; do
        is_installed "$unit" && { printf '%s' "$unit"; return 0; }
    done
    return 1
}

ensure_time_sync() {
    local unit
    unit=$(time_unit) || { ensure_packages chrony; unit=chrony; }
    if ! systemctl is-enabled --quiet "$unit"; then
        systemctl enable "$unit"
        changed "$unit enabled"
    fi
    systemctl is-active --quiet "$unit" || systemctl start "$unit"
}

ensure_swap() {
    [[ -z $(swapon --noheadings --show) ]] || return 0
    [[ -f /swapfile ]] || fallocate -l "$SWAP_SIZE" /swapfile
    chmod 600 /swapfile
    mkswap /swapfile >/dev/null
    swapon /swapfile
    grep -qE '^/swapfile[[:space:]]' /etc/fstab || printf '/swapfile none swap sw 0 0\n' >>/etc/fstab
    changed "swap /swapfile $SWAP_SIZE"
}

ensure_docker() {
    # A Docker already running stays as it is; replacing it would stop the stack.
    command -v docker >/dev/null && return 0
    ensure_packages ca-certificates curl gnupg
    local key=/etc/apt/keyrings/docker.asc tmp
    install -d -m 0755 /etc/apt/keyrings
    tmp=$(mktemp)
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o "$tmp"
    grep -q "^fpr:*$DOCKER_KEY_FINGERPRINT:" <<<"$(gpg --show-keys --with-colons "$tmp")" \
        || { rm -f "$tmp"; die "Docker apt key fingerprint does not match $DOCKER_KEY_FINGERPRINT"; }
    install -m 0644 "$tmp" "$key"
    rm -f "$tmp"
    put_file /etc/apt/sources.list.d/docker.sources 644 root:root "Types: deb
URIs: https://download.docker.com/linux/ubuntu
Suites: $VERSION_CODENAME
Components: stable
Architectures: $(dpkg --print-architecture)
Signed-By: $key" || true
    APT_UPDATED=0
    ensure_packages docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
}

verify_sshd() {
    local who settings
    for who in root "$ADMIN_USER" "$DEPLOY_USER"; do
        # Match blocks only apply with a connection context, so each user is checked through one.
        settings=$(sshd -T -C "user=$who,host=probe.invalid,addr=203.0.113.10" 2>/dev/null) \
            || { bad "sshd -T fails for $who"; continue; }
        if grep -qx 'passwordauthentication no' <<<"$settings" \
            && grep -qx 'permitrootlogin no' <<<"$settings" \
            && grep -qx 'kbdinteractiveauthentication no' <<<"$settings" \
            && grep -qx 'pubkeyauthentication yes' <<<"$settings"; then
            ok "sshd for $who: keys only, no root login"
        else
            bad "sshd for $who still allows passwords or root login"
        fi
    done
}

verify_firewall() {
    local status port rows
    status=$(ufw status verbose)
    grep -q '^Status: active' <<<"$status" && ok "ufw active" || bad "ufw inactive"
    grep -q 'deny (incoming)' <<<"$status" && ok "ufw denies incoming by default" || bad "ufw default incoming is not deny"
    grep -qE '^22/tcp +(ALLOW|LIMIT) IN +Anywhere *$' <<<"$status" && ok "ufw admits 22/tcp" || bad "ufw has no 22/tcp rule from Anywhere"
    grep -qE "^$BESZEL_PORT/tcp +ALLOW IN +$BESZEL_HUB *$" <<<"$status" \
        && ok "ufw admits Beszel $BESZEL_PORT/tcp from $BESZEL_HUB" || bad "ufw has no Beszel rule from $BESZEL_HUB"
    for port in "${OBSOLETE_PORTS[@]}"; do
        rows=$(grep -E "(ALLOW|LIMIT)" <<<"$status" | grep -E "(^|[ ,])$port([,/ ]|$)" || true)
        [[ -z $rows ]] && ok "ufw admits nothing on $port" || bad "ufw still admits $port: $(tr -s ' ' <<<"$rows" | paste -sd ';')"
    done
}

verify_unattended_upgrades() {
    local dump
    dump=$(apt-config dump)
    grep -q '^APT::Periodic::Unattended-Upgrade "1";' <<<"$dump" \
        && grep -q '^Unattended-Upgrade::Automatic-Reboot "true";' <<<"$dump" \
        && grep -q "^Unattended-Upgrade::Automatic-Reboot-Time \"$REBOOT_TIME\";" <<<"$dump" \
        && ok "unattended-upgrades on, reboot at $REBOOT_TIME $(date +%Z)" || bad "unattended-upgrades not configured"
    systemctl is-enabled --quiet apt-daily-upgrade.timer && ok "apt-daily-upgrade.timer enabled" || bad "apt-daily-upgrade.timer disabled"
    [[ $(date +%Z) == UTC ]] || log "note    host clock is $(date +%Z), so $REBOOT_TIME is not 02:00 WIB"
}

verify_deploy_user() {
    local home groups loose
    id "$DEPLOY_USER" >/dev/null 2>&1 || { bad "no $DEPLOY_USER user"; return; }
    groups=$(id -nG "$DEPLOY_USER")
    grep -qwE 'sudo|admin|wheel' <<<"$groups" && bad "$DEPLOY_USER is in a privileged group: $groups" || ok "$DEPLOY_USER has no sudo"
    home=$(getent passwd "$DEPLOY_USER" | cut -d: -f6)
    if [[ -s $home/.ssh/authorized_keys ]]; then
        loose=$(grep -vE '^(#|$)' "$home/.ssh/authorized_keys" | grep -vc "^restrict,command=\"$DEPLOY_COMMAND\" " || true)
        ((loose == 0)) && ok "$DEPLOY_USER keys run only $DEPLOY_COMMAND" || bad "$DEPLOY_USER has $loose key(s) without the forced command"
    else
        log "note    $DEPLOY_USER has no key yet (DEPLOY_PUBKEY)"
    fi
}

verify_host() {
    local unit ports
    if systemd-detect-virt --container --quiet; then
        log "note    container: the clock comes from the host"
    else
        unit=$(time_unit) && systemctl is-active --quiet "$unit" && ok "time sync by $unit" || bad "no active time sync"
    fi
    [[ -n $(swapon --noheadings --show) ]] && ok "swap active" || bad "no swap"
    if [[ -f $ENV_FILE ]]; then
        [[ $(stat -c '%a' "$ENV_FILE") == 600 ]] && ok "$ENV_FILE mode 600" || bad "$ENV_FILE is not mode 600"
    else
        log "note    $ENV_FILE not present yet"
    fi
    if is_installed docker-ce; then
        ok "docker-ce from download.docker.com"
    elif command -v docker >/dev/null; then
        bad "docker is not the docker-ce package from download.docker.com"
    else
        bad "docker missing"
    fi
    if ports=$(docker ps --format '{{.Names}} {{.Ports}}' 2>/dev/null); then
        grep -E ':(5432|6379)->' <<<"$ports" && bad "a container publishes 5432 or 6379" || ok "no container publishes 5432 or 6379"
    else
        log "note    docker daemon not reachable; published ports not checked"
    fi
}

main() {
    local mode=converge
    case ${1:-} in
        --check) mode=check ;;
        "") ;;
        *) die "usage: provision_host.sh [--check]" ;;
    esac
    preflight
    exec 9>/run/rotaria-provision.lock
    flock -n 9 || die "another run holds /run/rotaria-provision.lock"
    if [[ $mode == converge ]]; then
        export DEBIAN_FRONTEND=noninteractive
        ensure_firewall
        ensure_sshd
        ensure_deploy_user
        ensure_app_dir
        ensure_unattended_upgrades
        ensure_time_sync
        ensure_swap
        ensure_docker
        if ((${#CHANGES[@]})); then log "== ${#CHANGES[@]} change(s)"; else log "== no change"; fi
    fi
    log "== verify"
    verify_sshd
    verify_firewall
    verify_unattended_upgrades
    verify_deploy_user
    verify_host
    if ((${#FAILURES[@]})); then
        log "== ${#FAILURES[@]} check(s) failed"
        exit 1
    fi
    log "== all checks pass"
}

# Read from /dev/null so apt and friends cannot swallow the rest of a script piped through `bash -s`.
main "$@" </dev/null
