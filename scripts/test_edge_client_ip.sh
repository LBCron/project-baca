#!/usr/bin/env bash
set -euo pipefail

# Runs Caddyfile.prod in Docker in front of a header-echo upstream named
# `server` and checks the client address the API would see: only the
# connector's fixed address names the client through CF-Connecting-IP; any
# other peer (another container on the tunnel subnet, the host through the
# bridge gateway, another private subnet) is the client itself; no other
# address header ever reaches the upstream. Needs Docker and curl.

cd "$(dirname "$0")/.."
CADDY_IMAGE=$(awk '/^FROM caddy:/ {print $2; exit}' Dockerfile.edge)
PY_IMAGE=$(awk '/^FROM python:/ {print $2; exit}' Dockerfile.worker)
TAG="rotaria-edge-ip-$$"
TUNNEL_SUBNET=10.203.54.0/29
CONNECTOR_IP=10.203.54.2
EDGE_IP=10.203.54.3
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

docker network create --subnet "$TUNNEL_SUBNET" "$TAG-tunnel" >/dev/null
docker network create --subnet "$OTHER_SUBNET" "$TAG-other" >/dev/null
docker network create "$TAG-app" >/dev/null
docker run -d --name "$TAG-server" --network "$TAG-app" --network-alias server \
    "$PY_IMAGE" python -c "$ECHO_SERVER" >/dev/null
docker run -d --name "$TAG-edge" --network "$TAG-app" -e TRUSTED_PROXY="$CONNECTOR_IP" \
    -v "$PWD/Caddyfile.prod:/etc/caddy/Caddyfile:ro" "$CADDY_IMAGE" >/dev/null
docker network connect --ip "$EDGE_IP" "$TAG-tunnel" "$TAG-edge"
docker network connect "$TAG-other" "$TAG-edge"

SPOOFED=(-H "CF-Connecting-IP: 1.2.3.4" -H "X-Forwarded-For: 6.6.6.6" -H "X-Real-IP: 7.7.7.7"
    -H "True-Client-IP: 8.8.8.8" -H "Forwarded: for=9.9.9.9")

# Prints the upstream's view of the request headers as JSON, sent from a
# fresh container on the given network (and address, when given) with every
# spoofable address header.
probe() {
    docker run --rm --network "$1" ${2:+--ip "$2"} "$CADDY_IMAGE" sh -c '
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

# The same request from the host itself, which reaches the edge's tunnel
# address through the bridge gateway, inside the tunnel subnet.
probe_host() {
    local gateway
    gateway=$(docker network inspect "$TAG-tunnel" -f '{{range .IPAM.Config}}{{.Gateway}}{{end}}')
    for _ in $(seq 1 50); do
        if out=$(curl -sf -m 3 "${SPOOFED[@]}" "http://$EDGE_IP/api/v1/probe"); then
            printf '%s\nPEER %s\n' "$out" "$gateway"
            return 0
        fi
        sleep 0.2
    done
    return 1
}

status=0
check "connector address uses CF-Connecting-IP" 1.2.3.4 "$(probe "$TAG-tunnel" "$CONNECTOR_IP")" || status=1
check "other peer on the tunnel subnet is the client" peer "$(probe "$TAG-tunnel" 10.203.54.4)" || status=1
check "host through the bridge gateway is the client" peer "$(probe_host)" || status=1
check "peer on another private subnet is the client" peer "$(probe "$TAG-other")" || status=1
check "peer on the app network is the client" peer "$(probe "$TAG-app")" || status=1
exit $status
