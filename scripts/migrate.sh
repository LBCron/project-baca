#!/usr/bin/env bash
set -euo pipefail

# Project Baca — Database Migration Runner
# Executes paired SQL migrations (.up.sql and .down.sql) against PostgreSQL 17.
# With MIGRATE_DATABASE_URL set it talks to that database with a local psql
# (production, the compose `migrate` service); otherwise to the dev compose
# Postgres. Each file runs in one transaction together with its
# schema_migrations row, under an advisory lock, so a failed file leaves
# nothing behind and concurrent runs apply each file once.

DB_USER="${DB_USER:-baca_user}"
DB_NAME="${DB_NAME:-project_baca_db}"
MIGRATIONS_DIR="${MIGRATIONS_DIR:-migrations}"
# Any fixed bigint; every runner of this schema must agree on it.
LOCK_KEY=7262417015

psql_cmd() {
    if [[ -n "${MIGRATE_DATABASE_URL:-}" ]]; then
        psql "$MIGRATE_DATABASE_URL" -X -v ON_ERROR_STOP=1 "$@"
    else
        docker compose exec -T postgres psql -U "$DB_USER" -d "$DB_NAME" -X -v ON_ERROR_STOP=1 "$@"
    fi
}

# Feeds stdin to psql as one transaction that first takes the migration lock.
locked_tx() {
    { echo "SELECT pg_advisory_xact_lock(${LOCK_KEY}) \\gset"; cat; } | psql_cmd -q -1 -f -
}

init_table() {
    locked_tx <<'SQL'
SET LOCAL client_min_messages = warning;
CREATE TABLE IF NOT EXISTS schema_migrations (
    version VARCHAR(100) PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
-- Default privileges hand every new table to the production app role; the
-- ledger stays the owner's (scripts/postgres_app_role.sh). No-op in dev.
DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_roles WHERE rolname = 'rotaria_app') THEN
        REVOKE ALL ON schema_migrations FROM rotaria_app;
    END IF;
END
$$;
SQL
}

migrate_up() {
    init_table
    echo "Checking migration files in ${MIGRATIONS_DIR}..."

    shopt -s nullglob
    local files=("${MIGRATIONS_DIR}"/*.up.sql)
    shopt -u nullglob

    if [[ ${#files[@]} -eq 0 ]]; then
        echo "No .up.sql migration files found."
        return 0
    fi

    local applied_count=0
    for file in "${files[@]}"; do
        local basename
        basename=$(basename "$file")
        local version="${basename%.up.sql}"

        # Checked again under the lock: another runner may have applied it
        # while this one waited.
        local out
        if ! out=$(locked_tx <<SQL
SELECT EXISTS (SELECT 1 FROM schema_migrations WHERE version = '${version}') AS applied \\gset
\\if :applied
\\echo Skipped: ${version} (already applied).
\\else
\\echo Applying: ${version}...
$(cat "$file")
;
INSERT INTO schema_migrations (version) VALUES ('${version}');
\\echo Applied: ${version}.
\\endif
SQL
); then
            echo "$out"
            echo "Failed: ${version}; nothing of it was applied." >&2
            exit 1
        fi
        echo "$out"
        if [[ "$out" == *"Applied: ${version}."* ]]; then
            applied_count=$((applied_count + 1))
        fi
    done

    echo "Completed. Total new migrations applied: ${applied_count}."
}

migrate_down() {
    init_table
    local last_version
    last_version=$(psql_cmd -t -A -c "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1;")

    if [[ -z "$last_version" ]]; then
        echo "No recorded migrations to roll back."
        return 0
    fi

    local down_file="${MIGRATIONS_DIR}/${last_version}.down.sql"
    if [[ ! -f "$down_file" ]]; then
        echo "Error: Rollback file '${down_file}' not found."
        exit 1
    fi

    echo "Rolling back: ${last_version}..."
    locked_tx <<SQL
$(cat "$down_file")
;
DELETE FROM schema_migrations WHERE version = '${last_version}';
SQL
    echo "Rolled back: ${last_version}."
}

migrate_status() {
    init_table
    echo "Schema Migration Status:"
    psql_cmd -c "SELECT version, applied_at FROM schema_migrations ORDER BY version ASC;"
}

case "${1:-up}" in
    up)
        migrate_up
        ;;
    down)
        migrate_down
        ;;
    status)
        migrate_status
        ;;
    *)
        echo "Usage: $0 {up|down|status}"
        exit 1
        ;;
esac
