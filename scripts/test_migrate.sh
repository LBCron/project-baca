#!/usr/bin/env bash
# Exercises scripts/migrate.sh the way production runs it (URL mode inside
# the Postgres image) against a throwaway database on the dev stack: a file
# that fails halfway leaves nothing, two concurrent runs both succeed and
# apply once, and a rerun applies nothing.
set -euo pipefail

DB_CONTAINER=project_baca_db
PROBE_DB=migrate_probe
IMAGE=$(docker inspect "$DB_CONTAINER" --format '{{.Config.Image}}')
NETWORK=$(docker inspect "$DB_CONTAINER" --format '{{range $name, $_ := .NetworkSettings.Networks}}{{$name}}{{end}}')
WORK=$(mktemp -d)

admin_sql() {
    docker exec -i "$DB_CONTAINER" psql -U baca_user -d project_baca_db -X -q -v ON_ERROR_STOP=1 -c "$1"
}

probe_value() {
    docker exec -i "$DB_CONTAINER" psql -U baca_user -d "$PROBE_DB" -X -t -A -c "$1"
}

cleanup() {
    admin_sql "DROP DATABASE IF EXISTS ${PROBE_DB} WITH (FORCE);" || true
    rm -rf "$WORK"
}
trap cleanup EXIT

run_migrate() {
    docker run --rm --network "$NETWORK" \
        -v "$PWD/scripts/migrate.sh:/work/scripts/migrate.sh:ro" \
        -v "$WORK/migrations:/work/migrations:ro" \
        -w /work \
        -e MIGRATE_DATABASE_URL="postgres://baca_user:baca_password@postgres:5432/${PROBE_DB}" \
        --entrypoint bash "$IMAGE" scripts/migrate.sh "$@"
}

fail() {
    echo "FAIL: $1" >&2
    exit 1
}

admin_sql "DROP DATABASE IF EXISTS ${PROBE_DB} WITH (FORCE);"
admin_sql "CREATE DATABASE ${PROBE_DB};"
mkdir -p "$WORK/migrations"

echo "Case: the second statement of a file fails"
cat > "$WORK/migrations/20000101000001_probe_half.up.sql" <<'SQL'
CREATE TABLE probe_first (id INT);
SELECT 1 / 0;
SQL
if run_migrate up; then
    fail "a failing migration exited 0"
fi
[[ $(probe_value "SELECT to_regclass('probe_first') IS NULL") == t ]] || fail "probe_first survived the failed file"
[[ $(probe_value "SELECT count(*) FROM schema_migrations") == 0 ]] || fail "the failed file left a schema_migrations row"
rm "$WORK/migrations/20000101000001_probe_half.up.sql"

echo "Case: two runs at once"
cat > "$WORK/migrations/20000101000002_probe_slow.up.sql" <<'SQL'
CREATE TABLE probe_slow (id INT);
DO $$ BEGIN PERFORM pg_sleep(2); END $$;
SQL
run_migrate up > "$WORK/run_a.log" 2>&1 &
pid_a=$!
run_migrate up > "$WORK/run_b.log" 2>&1 &
pid_b=$!
wait "$pid_a" || { cat "$WORK/run_a.log"; fail "concurrent run A failed"; }
wait "$pid_b" || { cat "$WORK/run_b.log"; fail "concurrent run B failed"; }
applied=$(cat "$WORK/run_a.log" "$WORK/run_b.log" | grep -c "^Applied: 20000101000002_probe_slow.$" || true)
[[ $applied == 1 ]] || fail "the file was applied ${applied} times"
[[ $(probe_value "SELECT count(*) FROM schema_migrations") == 1 ]] || fail "expected one schema_migrations row"

echo "Case: a rerun applies nothing"
run_migrate up | tee "$WORK/rerun.log"
grep -q "Total new migrations applied: 0." "$WORK/rerun.log" || fail "the rerun applied something"

echo "migrate.sh: all cases passed"
