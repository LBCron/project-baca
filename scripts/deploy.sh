#!/usr/bin/env bash
# Production deploy on VPS-2 (GUIDE section 8). scripts/provision_host.sh
# installs it root-owned as /usr/local/bin/rotaria-deploy, the forced command of
# the deploy key. Run as `deploy` it re-runs itself through the one sudoers rule
# that user has, so `deploy` needs neither a shell nor the docker group.
#
# Request, from .github/workflows/deploy.yml once it verified the attestations
# (SSH_ORIGINAL_COMMAND for the key, the first argument for root):
#   X.Y.Z server=sha256:<hex> worker=sha256:<hex> edge=sha256:<hex> backup=sha256:<hex>
#
# It checks out tag vX.Y.Z in the repository clone, pins every release image to
# its verified index digest in .env.deploy (read by `make prod-*` after
# .env.production), pulls, checks what was pulled, and takes a backup. Any
# failure up to there restores the previous checkout and pins, and no running
# container changes but a first start of Postgres, Redis, and the embedding
# model (fetched before it starts) or the model service recreated for a
# changed EMBEDDER_IMAGE. Then: the migrations, server and worker, the edge once
# the API is healthy, every other service whose configuration changed, and a
# prune of release images other than the current and the previous deploy.
set -euo pipefail

SELF=/usr/local/bin/rotaria-deploy
# Overrides exist for the test harness; sudo's env_reset drops them in production.
REPO_DIR=${ROTARIA_REPO_DIR:-/srv/rotaria/repo}
IMAGE_PREFIX=${ROTARIA_IMAGE_PREFIX:-ghcr.io/aprxty3/project-baca}
IMAGES=(server worker edge backup)
PINS=.env.deploy
WAIT=180
# Loading the 2.2 GB model: about 4 s on amd64, unmeasured on arm64 (GUIDE
# section 8). The embedder healthcheck's start_period (600 s) is the load
# budget; this covers it plus three failed checks.
EMBEDDER_WAIT=${ROTARIA_EMBEDDER_WAIT:-660}

log() { printf '== %s\n' "$*"; }
die() { printf 'rotaria-deploy: %s\n' "$*" >&2; exit 1; }

# Sets VERSION and DIGEST[image] from a request, or dies naming the problem.
parse_request() {
    local request=$1 word name
    local -a words
    local digest='sha256:[0-9a-f]{64}'
    [[ $request =~ ^(0|[1-9][0-9]{0,5})\.(0|[1-9][0-9]{0,5})\.(0|[1-9][0-9]{0,5})(\ [a-z]+=$digest){${#IMAGES[@]}}$ ]] \
        || die "expected 'X.Y.Z server=sha256:<64 hex> worker=... edge=... backup=...'"
    read -r -a words <<<"$request"
    VERSION=${words[0]}
    declare -gA DIGEST=()
    for word in "${words[@]:1}"; do
        name=${word%%=*}
        [[ " ${IMAGES[*]} " == *" $name "* ]] || die "unknown image $name"
        [[ -z ${DIGEST[$name]:-} ]] || die "image $name given twice"
        DIGEST[$name]=${word#*=}
    done
}

git_repo() { git -c safe.directory="$REPO_DIR" -C "$REPO_DIR" "$@"; }

compose() { docker compose --env-file .env.production --env-file "$1" -f docker-compose.prod.yml "${@:2}"; }

image_ref() { printf '%s/%s:%s@%s' "$IMAGE_PREFIX" "$1" "$VERSION" "${DIGEST[$1]}"; }

restore() {
    log "restoring the previous checkout; the running stack is unchanged"
    git_repo checkout -q "$PREV_REF" || true
    rm -f "$PINS.next"
}

after_change() {
    printf 'rotaria-deploy: failed after the stack began to change; dispatch the previous version to roll back\n' >&2
}

checkout() {
    PREV_REF=$(git_repo rev-parse HEAD)
    trap restore EXIT
    log "checking out v$VERSION"
    git_repo fetch -q --depth 1 origin tag "v$VERSION"
    git_repo checkout -q "v$VERSION"
}

pin_and_pull() {
    local name expected
    {
        printf 'ROTARIA_VERSION=%s\n' "$VERSION"
        for name in "${IMAGES[@]}"; do
            printf 'ROTARIA_%s_IMAGE=%s\n' "${name^^}" "$(image_ref "$name")"
        done
    } >"$PINS.next"
    # A tag whose compose file ignores the pins would run something unverified.
    expected=$(compose "$PINS.next" config --images)
    for name in "${IMAGES[@]}"; do
        grep -qxF "$(image_ref "$name")" <<<"$expected" || die "docker-compose.prod.yml of v$VERSION does not run $(image_ref "$name")"
    done
    log "pulling"
    compose "$PINS.next" pull --quiet
    for name in "${IMAGES[@]}"; do
        docker image inspect --format '{{join .RepoDigests "\n"}}' "$(image_ref "$name")" \
            | grep -qxF "$IMAGE_PREFIX/$name@${DIGEST[$name]}" \
            || die "pulled $name does not carry the verified digest ${DIGEST[$name]}"
    done
}

# The running pins become the rollback target only once this deploy succeeds;
# re-dispatching the running version keeps the one before it.
commit_pins() {
    if [[ -f $PINS ]] && ! cmp -s "$PINS" "$PINS.next"; then
        cp "$PINS" "$PINS.prev.pending"
    fi
    mv "$PINS.next" "$PINS"
    trap after_change EXIT
}

rotate_prev() {
    if [[ -f $PINS.prev.pending ]]; then
        mv "$PINS.prev.pending" "$PINS.prev"
    fi
}

check_api() {
    compose "$PINS" exec -T server wget -q -O /dev/null http://localhost:8080/api/v1/health \
        || die "server answers no healthy /api/v1/health"
}

check_web() {
    compose "$PINS" exec -T edge wget -q -O /dev/null http://127.0.0.1/ || die "edge serves no web page"
    compose "$PINS" exec -T edge wget -q -O /dev/null http://127.0.0.1/api/v1/health || die "edge reaches no healthy API"
}

# Keeps the images of the current and the previous deploy, for a fast rollback.
prune_images() {
    local file ref name id
    local -A keep=()
    for file in "$PINS" "$PINS.prev"; do
        [[ -f $file ]] || continue
        while IFS='=' read -r name ref; do
            [[ $name == ROTARIA_*_IMAGE ]] || continue
            id=$(docker image inspect --format '{{.Id}}' "$ref" 2>/dev/null) && keep[$id]=1
        done <"$file"
    done
    ((${#keep[@]} >= ${#IMAGES[@]})) || die "cannot resolve the current images; nothing pruned"
    for name in "${IMAGES[@]}"; do
        while read -r id; do
            [[ -n $id && -z ${keep[$id]:-} ]] || continue
            log "removing old $name image $id"
            docker image rm -f "$id" >/dev/null || true
        done < <(docker image ls --no-trunc --quiet "$IMAGE_PREFIX/$name" | sort -u)
    done
}

deploy() {
    cd "$REPO_DIR"
    checkout
    pin_and_pull
    # Started on a first deploy; a changed Postgres or Redis waits for the last step.
    compose "$PINS.next" up -d --no-recreate --wait --wait-timeout "$WAIT" postgres redis
    # The embedding model before anything needs it: the first deploy downloads
    # it (about 2.2 GB), later ones find it in its volume. A changed
    # EMBEDDER_IMAGE recreates the service here, so the reload is checked
    # too (quote search answers 503 while it loads). A failed download or a
    # model that never turns healthy stops here, before the pins move.
    log "embedding model"
    compose "$PINS.next" run --rm --no-deps -T embedder-fetch
    compose "$PINS.next" up -d --no-deps --wait --wait-timeout "$EMBEDDER_WAIT" embedder
    # Before the pins move: a failed backup leaves checkout and pins as they were.
    log "backup before migrating"
    compose "$PINS.next" run --rm --no-deps -T backup pg_backup.sh
    commit_pins
    log "migrating"
    make --no-print-directory prod-migrate
    log "server and worker"
    compose "$PINS" up -d --no-deps --wait --wait-timeout "$WAIT" server worker
    check_api
    log "edge"
    compose "$PINS" up -d --no-deps --wait --wait-timeout "$WAIT" edge
    check_web
    log "every other service whose configuration changed"
    local out
    out=$(compose "$PINS" up -d --remove-orphans 2>&1) || { printf '%s\n' "$out" >&2; die "compose up failed"; }
    grep -E 'Recreated|Created|Started|Removed' <<<"$out" || echo "nothing else changed"
    rotate_prev
    log "pruning old release images"
    prune_images
    trap - EXIT
    compose "$PINS" ps --format 'table {{.Service}}\t{{.Image}}\t{{.Status}}'
    log "deployed $VERSION"
}

main() {
    if ((EUID != 0)); then
        exec sudo -n "$SELF" "${SSH_ORIGINAL_COMMAND:-}"
    fi
    parse_request "${1:-}"
    exec 9>/run/rotaria-deploy.lock
    flock -n 9 || die "another deploy is running"
    deploy
}

# Sourced by scripts/test_deploy.sh for parse_request alone.
if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
    main "$@"
fi
