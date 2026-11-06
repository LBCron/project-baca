#!/usr/bin/env bash
# Dev controller behind `make dev backend|frontend|stop`. One machine has one
# API port and one web port, shared by every checkout and worktree, so this
# script owns them: it records which checkout runs each service, takes the
# port over from another checkout (or a manual `make dev-server`/`dev-web` of
# this repo), and returns only once the service answers.
set -euo pipefail

ROOT=$(git rev-parse --show-toplevel)
MAIN=$(cd "$(git rev-parse --git-common-dir)/.." && pwd)
STATE=${ROTARIA_DEV_STATE:-${XDG_STATE_HOME:-$HOME/.local/state}/rotaria-dev}
mkdir -p "$STATE"
cd "$ROOT"
# Worktrees share the main checkout's build cache instead of a cold target/.
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$MAIN/target}

say() { printf 'dev: %s\n' "$*" >&2; }
die() {
	say "$*"
	exit 1
}

# A new worktree has no .env; the main checkout's is the dev one.
ensure_env() {
	[[ -e .env ]] && return
	[[ -f $MAIN/.env ]] || die "no .env in $ROOT or $MAIN; copy .env.example first"
	ln -s "$MAIN/.env" .env
	say "linked .env -> $MAIN/.env"
}

port_of() {
	case $1 in
	backend) echo "${APP_PORT:-$(sed -n 's/^APP_PORT=//p' .env | tail -n 1)}" ;;
	frontend) echo 3000 ;;
	esac
}

listener_pid() { ss -Hltnp "sport = :$1" | grep -o 'pid=[0-9]*' | head -n 1 | cut -d= -f2 || true; }

# Only processes started from this repository may be taken over.
ours() {
	local cwd
	cwd=$(readlink "/proc/$1/cwd" 2>/dev/null) || return 1
	[[ $cwd == "$MAIN" || $cwd == "$MAIN"/* ]]
}

stop_group() {
	local pgid=$1
	kill -TERM -- "-$pgid" 2>/dev/null || return 0
	for _ in $(seq 50); do
		kill -0 -- "-$pgid" 2>/dev/null || return 0
		sleep 0.2
	done
	kill -KILL -- "-$pgid" 2>/dev/null || true
}

stop_service() {
	local svc=$1 port pid pgid
	port=$(port_of "$svc")
	if [[ -f $STATE/$svc.pgid ]]; then
		stop_group "$(cat "$STATE/$svc.pgid")"
		rm -f "$STATE/$svc.pgid" "$STATE/$svc.owner"
	fi
	pid=$(listener_pid "$port")
	[[ -z $pid ]] && return
	ours "$pid" || die "port $port is held by pid $pid ($(cat "/proc/$pid/comm" 2>/dev/null)) outside this repo; stop it yourself"
	pgid=$(ps -o pgid= -p "$pid" | tr -d ' ')
	say "taking $svc port $port over from pid $pid (process group $pgid) in $(readlink "/proc/$pid/cwd")"
	stop_group "$pgid"
}

wait_ready() {
	local svc=$1 url=$2 limit=$3 pgid start
	pgid=$(cat "$STATE/$svc.pgid")
	start=$SECONDS
	until curl -fsS -o /dev/null --max-time 2 "$url" 2>/dev/null; do
		if ! kill -0 -- "-$pgid" 2>/dev/null; then
			tail -n 40 "$STATE/$svc.log" >&2
			die "$svc exited before answering $url (log: $STATE/$svc.log)"
		fi
		((SECONDS - start < limit)) || die "$svc did not answer $url within ${limit}s (log: $STATE/$svc.log)"
		sleep 1
	done
	say "$svc ready at $url from $ROOT ($((SECONDS - start))s, log: $STATE/$svc.log)"
}

# setsid gives the service its own process group, so a stop takes cargo,
# the server, trunk, and its build children down together.
launch() {
	local svc=$1
	shift
	: >"$STATE/$svc.log"
	# The service must not inherit the controller lock, or it would hold it for life.
	setsid "$@" >>"$STATE/$svc.log" 2>&1 </dev/null {LOCK_FD}>&- &
	echo $! >"$STATE/$svc.pgid"
	echo "$ROOT" >"$STATE/$svc.owner"
}

# Starts what the API needs without recreating containers whose image moved.
# Any SMTP catcher on the dev port will do (another project's Mailpit often
# holds it), so mailpit starts only when the port is free.
services() {
	local svcs=(postgres redis minio)
	[[ -n $(listener_pid "$(sed -n 's/^SMTP_PORT=//p' .env | tail -n 1)") ]] || svcs+=(mailpit)
	COMPOSE_PROJECT_NAME=project-baca docker compose up -d --wait --no-recreate "${svcs[@]}" >/dev/null
}

# The server reads .env at boot and does not reload, so every call restarts it
# from this checkout's code.
backend() {
	ensure_env
	services
	stop_service backend
	launch backend cargo run -q -p server
	wait_ready backend "http://127.0.0.1:$(port_of backend)/api/v1/health" "${DEV_BACKEND_TIMEOUT:-900}"
}

# Trunk hot-reloads its own checkout, so an instance already serving it is kept.
frontend() {
	ensure_env
	if [[ -f $STATE/frontend.pgid && $(cat "$STATE/frontend.owner" 2>/dev/null) == "$ROOT" ]] &&
		kill -0 -- "-$(cat "$STATE/frontend.pgid")" 2>/dev/null; then
		say "frontend already serving $ROOT on :3000 (log: $STATE/frontend.log)"
		return
	fi
	stop_service frontend
	cd crates/web
	launch frontend trunk serve
	cd "$ROOT"
	wait_ready frontend "http://127.0.0.1:3000/" "${DEV_FRONTEND_TIMEOUT:-1200}"
}

# One controller run per service at a time, across all checkouts.
locked() {
	exec {LOCK_FD}>"$STATE/$1.lock"
	flock "$LOCK_FD"
	"$2" "$1"
	exec {LOCK_FD}>&-
}

(($#)) || die "usage: make dev backend|frontend|stop"
for cmd in "$@"; do
	case $cmd in
	backend) locked backend backend ;;
	frontend) locked frontend frontend ;;
	stop)
		locked backend stop_service
		locked frontend stop_service
		say "backend and frontend stopped"
		;;
	*) die "unknown service '$cmd'; use backend, frontend, or stop" ;;
	esac
done
