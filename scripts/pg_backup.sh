#!/usr/bin/env bash
# Encrypted Postgres backups in the backup bucket, run inside the backup image
# (Dockerfile.backup) by the compose `backup` service and `make prod-backup`.
#
#   pg_backup.sh                      one dump now
#   pg_backup.sh --every SECONDS      one dump now, then one per period; a failed
#                                     run is logged and the loop goes on
#   pg_backup.sh --latest             prints the newest object key
#   pg_backup.sh --decrypt KEY FILE   streams that object, decrypted with the age
#                                     identity FILE, to stdout (restore drill)
#
# A dump is `pg_dump -Fc` of PGDATABASE streamed through age to the owner's
# public key and into BACKUP_S3_BUCKET at pg/YYYY/MM/DD/pg_<stamp>.dump.age; the
# plaintext never touches a disk. After a stored dump it pings
# BACKUP_HEARTBEAT_URL (a Healthchecks.io check) when that is set.
# Environment: PGHOST, PGUSER, PGPASSWORD, PGDATABASE; BACKUP_AGE_RECIPIENT,
# BACKUP_S3_ENDPOINT, BACKUP_S3_ACCESS_KEY_ID, BACKUP_S3_SECRET_ACCESS_KEY,
# BACKUP_S3_BUCKET, BACKUP_S3_PROVIDER (rclone provider, default Cloudflare),
# BACKUP_S3_REGION (default auto), BACKUP_HEARTBEAT_URL.
set -euo pipefail

log() { printf 'pg_backup: %s\n' "$*" >&2; }
die() { log "$*"; exit 1; }

: "${BACKUP_S3_ENDPOINT:?set BACKUP_S3_ENDPOINT}" "${BACKUP_S3_BUCKET:?set BACKUP_S3_BUCKET}"
: "${BACKUP_S3_ACCESS_KEY_ID:?set BACKUP_S3_ACCESS_KEY_ID}" "${BACKUP_S3_SECRET_ACCESS_KEY:?set BACKUP_S3_SECRET_ACCESS_KEY}"

# The rclone remote `backup` lives in the environment only, never in a file.
export RCLONE_CONFIG_BACKUP_TYPE=s3
export RCLONE_CONFIG_BACKUP_PROVIDER=${BACKUP_S3_PROVIDER:-Cloudflare}
export RCLONE_CONFIG_BACKUP_ENDPOINT=$BACKUP_S3_ENDPOINT
export RCLONE_CONFIG_BACKUP_REGION=${BACKUP_S3_REGION:-auto}
export RCLONE_CONFIG_BACKUP_ACCESS_KEY_ID=$BACKUP_S3_ACCESS_KEY_ID
export RCLONE_CONFIG_BACKUP_SECRET_ACCESS_KEY=$BACKUP_S3_SECRET_ACCESS_KEY
# The token reaches one bucket and may not create or inspect buckets.
export RCLONE_CONFIG_BACKUP_NO_CHECK_BUCKET=true
BUCKET=backup:$BACKUP_S3_BUCKET

dump_once() {
    : "${PGDATABASE:?set PGDATABASE}" "${BACKUP_AGE_RECIPIENT:?set BACKUP_AGE_RECIPIENT}"
    local stamp key count
    stamp=$(date -u +%Y%m%dT%H%M%SZ)
    key="pg/${stamp:0:4}/${stamp:4:2}/${stamp:6:2}/pg_${stamp}.dump.age"
    # A wrong database name or password stops here, before any object exists.
    psql -X -q -t -c 'SELECT 1' >/dev/null || die "cannot reach database ${PGDATABASE}; nothing written"
    count=$(mktemp)
    # dd passes the dump through and reports its size, so an empty dump is caught.
    if ! pg_dump --format=custom | dd bs=65536 2>"$count" | age -r "$BACKUP_AGE_RECIPIENT" \
        | rclone rcat "$BUCKET/$key"; then
        rclone deletefile "$BUCKET/$key" 2>/dev/null || true
        rm -f "$count"
        die "dump or upload of $key failed; removed the partial object"
    fi
    local bytes
    bytes=$(awk '/bytes/ { print $1 }' "$count")
    rm -f "$count"
    if [[ ! $bytes =~ ^[0-9]+$ ]] || ((bytes == 0)); then
        rclone deletefile "$BUCKET/$key" 2>/dev/null || true
        die "empty dump for $key; removed it"
    fi
    log "stored $BACKUP_S3_BUCKET/$key ($bytes bytes before encryption)"
    if [[ -n ${BACKUP_HEARTBEAT_URL:-} ]]; then
        wget -q -O /dev/null -T 10 "$BACKUP_HEARTBEAT_URL" || log "heartbeat ping failed; the dump is stored"
    fi
}

every() {
    local period=$1
    [[ $period =~ ^[1-9][0-9]*$ ]] || die "--every takes a period in seconds"
    while true; do
        "$0" || log "run failed; next try in ${period}s"
        sleep "$period"
    done
}

latest() {
    local newest
    # Keys sort by date and stamp, so the last one is the newest.
    newest=$(rclone lsf -R --files-only "$BUCKET/pg" | grep '\.dump\.age$' | sort | tail -n 1)
    [[ -n $newest ]] || die "no dump under $BACKUP_S3_BUCKET/pg"
    printf 'pg/%s\n' "$newest"
}

decrypt() {
    local key=$1 identity=$2
    [[ $key =~ ^pg/[0-9]{4}/[0-9]{2}/[0-9]{2}/pg_[0-9]{8}T[0-9]{6}Z\.dump\.age$ ]] || die "not a dump key: $key"
    [[ -r $identity ]] || die "cannot read the age identity $identity"
    rclone cat "$BUCKET/$key" | age -d -i "$identity"
}

case ${1:-} in
    "") dump_once ;;
    --every) every "${2:-}" ;;
    --latest) latest ;;
    --decrypt) decrypt "${2:?--decrypt KEY IDENTITY_FILE}" "${3:?--decrypt KEY IDENTITY_FILE}" ;;
    *) die "usage: pg_backup.sh [--every SECONDS | --latest | --decrypt KEY IDENTITY_FILE]" ;;
esac
