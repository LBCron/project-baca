#!/usr/bin/env bash
# Sends GET URL at RATE requests per second for DURATION seconds and sums up
# what came back, to show a deploy drops nothing (make deploy-probe). Each
# request is one curl started on schedule, so a slow answer never delays the
# next. With CF_ACCESS_CLIENT_ID and CF_ACCESS_CLIENT_SECRET set, both go along
# as the Cloudflare Access service token headers, read by curl from a file so
# they never show in `ps`. Exits 1 when any request failed, answered 5xx or
# 429, or got no answer.
#
# Usage: deploy_probe.sh URL RATE DURATION
set -euo pipefail

url=${1:-} rate=${2:-} duration=${3:-}
[[ $url =~ ^https?:// && $rate =~ ^[1-9][0-9]*$ && $duration =~ ^[1-9][0-9]*$ ]] \
    || { echo "usage: deploy_probe.sh URL RATE DURATION   (rate per second, duration in seconds)" >&2; exit 2; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
headers=$work/headers
(umask 077 && : >"$headers")
if [[ -n ${CF_ACCESS_CLIENT_ID:-} && -n ${CF_ACCESS_CLIENT_SECRET:-} ]]; then
    printf 'CF-Access-Client-Id: %s\nCF-Access-Client-Secret: %s\n' \
        "$CF_ACCESS_CLIENT_ID" "$CF_ACCESS_CLIENT_SECRET" >"$headers"
fi

# One line per request: HTTP status (000 when no response came) and seconds.
probe() {
    curl -s -o /dev/null -H @"$headers" --max-time 30 -w '%{http_code} %{time_total}\n' "$url" \
        >"$work/r$1" 2>/dev/null || true
}

now_us() { printf '%s' "${EPOCHREALTIME/[.,]/}"; }

total=$((rate * duration))
step=$((1000000 / rate))
start=$(now_us)
echo "probing $url: $rate/s for ${duration}s ($total requests)"
for ((i = 0; i < total; i++)); do
    wait_us=$((start + i * step - $(now_us)))
    if ((wait_us > 0)); then
        sleep "$(printf '%d.%06d' $((wait_us / 1000000)) $((wait_us % 1000000)))"
    fi
    probe "$i" &
done
wait

# Sorted by time, so the median is the middle line; plain awk, no gawk extensions.
cat "$work"/r* | sort -k2,2n | awk -v total="$total" '
    { n++; t[n] = $2; status[$1]++; if ($1 ~ /^5/) five++ }
    END {
        for (s in status) printf "status %s: %d\n", (s == "000" ? "000 (no response)" : s), status[s]
        median = n % 2 ? t[(n + 1) / 2] : (t[n / 2] + t[n / 2 + 1]) / 2
        printf "requests %d of %d; connection errors %d, 5xx %d, 429 %d\n", n, total, status["000"], five, status["429"]
        printf "latency median %.3f s, max %.3f s\n", median, t[n]
        exit (n != total || status["000"] || five || status["429"]) ? 1 : 0
    }'
