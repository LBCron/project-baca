#!/usr/bin/env bash
# Project Baca — PostgreSQL logical backup (dev/prod).
# Usage: ./scripts/pg_backup.sh [postgres://user:pass@host:port/db]
# Output: backups/pg_<db>_<timestamp>.dump (custom format; restore with pg_restore).
# Prod cron example (daily 02:00, keep 7): 0 2 * * * /srv/baca/scripts/pg_backup.sh >> /var/log/baca-backup.log 2>&1
set -euo pipefail

DB_URL="${1:-${DATABASE_URL:-postgres://baca_user:baca_password@localhost:5433/project_baca_db}}"
OUT_DIR="$(dirname "$0")/../backups"
mkdir -p "$OUT_DIR"

DB_NAME="$(echo "$DB_URL" | sed -E 's#.*/([^/?]+).*#\1#')"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="$OUT_DIR/pg_${DB_NAME}_${STAMP}.dump"

if command -v pg_dump >/dev/null 2>&1; then
    pg_dump --format=custom --file="$OUT" "$DB_URL"
else
    docker exec project_baca_db pg_dump -U baca_user -d project_baca_db -Fc > "$OUT"
fi

ls -lh "$OUT"
echo "Verify restore any time with: pg_restore --list $OUT | head"
