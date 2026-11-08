#!/usr/bin/env bash
# The request check of scripts/deploy.sh: the only input the deploy key can
# send. Runs without Docker, root, or a host.
set -euo pipefail
cd "$(dirname "$0")"
# shellcheck source-path=SCRIPTDIR
source ./deploy.sh

H=$(printf 'a%.0s' {1..64})
D=sha256:$H
OK="1.2.3 server=$D worker=$D edge=$D backup=$D"
failures=0

expect() {
    local want=$1 label=$2 request=$3 got=accepted
    (parse_request "$request") >/dev/null 2>&1 || got=refused
    if [[ $got == "$want" ]]; then
        printf 'ok    %-8s %s\n' "$want" "$label"
    else
        printf 'FAIL  %-8s %s (got %s)\n' "$want" "$label" "$got"
        failures=$((failures + 1))
    fi
}

expect accepted 'four images in order' "$OK"
expect accepted 'any order, two-digit minor' "0.10.0 backup=$D edge=$D worker=$D server=$D"
expect refused 'empty request' ""
expect refused 'version only' "1.2.3"
expect refused 'leading v' "v1.2.3 server=$D worker=$D edge=$D backup=$D"
expect refused 'leading zero' "01.2.3 server=$D worker=$D edge=$D backup=$D"
expect refused 'pre-release' "1.2.3-rc.1 server=$D worker=$D edge=$D backup=$D"
expect refused 'two-part version' "1.2 server=$D worker=$D edge=$D backup=$D"
expect refused 'an image missing' "1.2.3 server=$D worker=$D edge=$D"
expect refused 'a fifth image' "$OK web=$D"
expect refused 'an image twice' "1.2.3 server=$D server=$D edge=$D backup=$D"
expect refused 'unknown image' "1.2.3 server=$D worker=$D edge=$D redis=$D"
expect refused 'uppercase hex' "1.2.3 server=sha256:${H^^} worker=$D edge=$D backup=$D"
expect refused '63 hex digits' "1.2.3 server=sha256:${H:1} worker=$D edge=$D backup=$D"
expect refused 'other algorithm' "1.2.3 server=sha512:$H worker=$D edge=$D backup=$D"
expect refused 'double space' "1.2.3  server=$D worker=$D edge=$D backup=$D"
expect refused 'leading space' " $OK"
expect refused 'trailing space' "$OK "
expect refused 'second line' "$OK"$'\n'"rm -rf /"
expect refused 'semicolon' "1.2.3 server=$D;id worker=$D edge=$D backup=$D"
expect refused 'command substitution' "1.2.3 server=\$(id) worker=$D edge=$D backup=$D"
expect refused 'tab separator' "1.2.3"$'\t'"server=$D worker=$D edge=$D backup=$D"

(
    parse_request "$OK"
    [[ $VERSION == 1.2.3 && ${DIGEST[server]} == "$D" && ${DIGEST[backup]} == "$D" ]]
) || { echo "FAIL  parsed values"; failures=$((failures + 1)); }

if ((failures)); then
    echo "$failures case(s) failed"
    exit 1
fi
echo "all request cases pass"
