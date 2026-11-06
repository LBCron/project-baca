#!/usr/bin/env bash
set -euo pipefail

# Runs Caddyfile.prod in Docker in front of a header-echo upstream named
# `server` and checks the client address the API would see: a peer on the
# trusted tunnel subnet names the client through CF-Connecting-IP; any other
# peer, one on another private subnet included, is the client itself; no
# other address header ever reaches the upstream. Needs only Docker.

cd "$(dirname "$0")/.."
CADDY_IMAGE=$(awk '/^FROM caddy:/ {print $2; exit}' Dockerfile.edge)
PY_IMAGE=$(awk '/^FROM python:/ {print $2; exit}' Dockerfile.worker)
TAG="rotaria-edge-ip-$$"
TRUSTED_SUBNET=10.203.54.0/29
OTHER_SUBNET=10.203.55.0/29

cleanup() {
    docker rm -f "$TAG-edge" "$TAG-server" >/dev/null 2>&1 || true
    docker network rm "$TAG-tunnel" "$TAG-other" "$TAG-app" >/dev/null 2>&1 || true
}
trap cleanup EXIT

ECHO_SERVER='
import json, http.server
class H(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        body = json.dumps({k.lower(): v for k, v in self.headers.items()}).encode()
        self.send_response(200); self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body))); self.end_headers(); self.wfile.write(body)
    def log_message(self, *a): pass
http.server.ThreadingHTTPServer(("0.0.0.0", 8080), H).serve_forever()
'

docker network create --subnet "$TRUSTED_SUBNET" "$TAG-tunnel" >/dev/null
docker network create --subnet "$OTHER_SUBNET" "$TAG-other" >/dev/null
docker network create "$TAG-app" >/dev/null
docker run -d --name "$TAG-server" --network "$TAG-app" --network-alias server \
    "$PY_IMAGE" python -c "$ECHO_SERVER" >/dev/null
docker run -d --name "$TAG-edge" --network "$TAG-app" -e TRUSTED_PROXY_SUBNET="$TRUSTED_SUBNET" \
    -v "$PWD/Caddyfile.prod:/etc/caddy/Caddyfile:ro" "$CADDY_IMAGE" >/dev/null
docker network connect "$TAG-tunnel" "$TAG-edge"
docker network connect "$TAG-other" "$TAG-edge"

# Prints the upstream's view of the request headers as JSON, sent from a
# fresh container on the given network with every spoofable address header.
probe() {
    docker run --rm --network "$1" "$CADDY_IMAGE" sh -c '
        for i in $(seq 1 50); do
            out=$(wget -qO- -T 3 \
                --header "CF-Connecting-IP: 1.2.3.4" \
                --header "X-Forwarded-For: 6.6.6.6" \
                --header "X-Real-IP: 7.7.7.7" \
                --header "True-Client-IP: 8.8.8.8" \
                --header "Forwarded: for=9.9.9.9" \
                http://'"$TAG"'-edge/api/v1/probe 2>/dev/null) && { echo "$out"; echo "PEER $(hostname -i)"; exit 0; }
            sleep 0.2
        done
        exit 1'
}

check() {
    python3 -I -c '
import json, sys
label, want, text = sys.argv[1], sys.argv[2], sys.argv[3]
lines = text.strip().splitlines()
headers, peer = json.loads(lines[0]), lines[1].split()[1]
expected = peer if want == "peer" else want
xff = headers.get("x-forwarded-for")
leaked = [h for h in ("cf-connecting-ip", "x-real-ip", "true-client-ip", "forwarded") if h in headers]
ok = xff == expected and not leaked
result = "PASS" if ok else "FAIL"
print(f"{result} {label}: X-Forwarded-For={xff} expected={expected} leaked={leaked}")
sys.exit(0 if ok else 1)
' "$1" "$2" "$3"
}

status=0
check "trusted tunnel peer uses CF-Connecting-IP" 1.2.3.4 "$(probe "$TAG-tunnel")" || status=1
check "peer on another private subnet is the client" peer "$(probe "$TAG-other")" || status=1
check "peer on the app network is the client" peer "$(probe "$TAG-app")" || status=1
exit $status
