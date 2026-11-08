#!/usr/bin/env bash
# Restore drill (make prod-restore-drill): the newest dump in the backup bucket,
# or OBJECT, decrypted with the owner's age identity and restored with
# --exit-on-error into a throwaway pgvector/pgvector:pg17 container. That
# container gets rotaria_app from scripts/postgres_app_role.sh at init, as a
# new production volume does, before the dump's grants to it arrive. Prints the
# row counts, the last migration, and how long the drill took; removes the
# container and its data volume on exit. The dump only crosses this host as a
# pipe.
#
# Environment: PROD_COMPOSE (the Makefile's compose command), AGE_IDENTITY (the
# owner's age private key file), OBJECT (optional key, pg/YYYY/MM/DD/<name>).
set -euo pipefail

: "${PROD_COMPOSE:?run through make prod-restore-drill}"
: "${AGE_IDENTITY:?set AGE_IDENTITY to the age private key file}"
[[ -r $AGE_IDENTITY ]] || { echo "restore drill: cannot read $AGE_IDENTITY" >&2; exit 2; }
identity=$(realpath "$AGE_IDENTITY")
read -ra compose <<<"$PROD_COMPOSE"
started=$SECONDS

# The backup service's own image and bucket settings, without starting Postgres.
backup() {
    "${compose[@]}" run --rm --no-deps -T --user "$(id -u):$(id -g)" \
        -v "$identity:/run/age-identity:ro" backup pg_backup.sh "$@"
}

image=$("${compose[@]}" config --images | grep -m1 '^pgvector/pgvector:')
object=${OBJECT:-$(backup --latest)}
echo "Restoring $object"

pg=rotaria-restore-drill-$$
# -v: the image declares a data volume, which would keep the restored copy on disk.
cleanup() { docker rm -fv "$pg" >/dev/null 2>&1 || true; }
trap cleanup EXIT
POSTGRES_PASSWORD=$(od -An -N16 -tx1 /dev/urandom | tr -d ' \n')
POSTGRES_APP_PASSWORD=$(od -An -N16 -tx1 /dev/urandom | tr -d ' \n')
export POSTGRES_PASSWORD POSTGRES_APP_PASSWORD
# Capped so a drill on the production host cannot starve production Postgres.
docker run -d --name "$pg" --network none --memory 1g --cpus 1 \
    -e POSTGRES_USER=baca_user -e POSTGRES_DB=project_baca_db -e POSTGRES_PASSWORD -e POSTGRES_APP_PASSWORD \
    -v "$PWD/scripts/postgres_app_role.sh:/docker-entrypoint-initdb.d/10-app-role.sh:ro" \
    "$image" >/dev/null

# The init server listens on the socket only, so TCP answers once init is done.
for _ in $(seq 120); do
    docker exec "$pg" pg_isready -q -h 127.0.0.1 -U baca_user -d project_baca_db && break
    sleep 1
done
psql() { docker exec -i "$pg" psql -X -A -t -q -v ON_ERROR_STOP=1 -U baca_user -d project_baca_db "$@"; }
[[ $(psql -c "SELECT count(*) FROM pg_roles WHERE rolname = 'rotaria_app'") == 1 ]] \
    || { echo "restore drill: rotaria_app missing before the restore" >&2; exit 1; }

backup --decrypt "$object" /run/age-identity \
    | docker exec -i "$pg" pg_restore --exit-on-error -U baca_user -d project_baca_db

psql -F ' ' -c "SELECT (SELECT count(*) FROM users), (SELECT count(*) FROM books),
    (SELECT count(*) FROM book_chunks), (SELECT max(version) FROM schema_migrations)" \
    | { read -r users books chunks migration
        printf 'users        %s\nbooks        %s\nbook_chunks  %s\nmigration    %s\n' "$users" "$books" "$chunks" "$migration"; }
echo "Restore drill took $((SECONDS - started)) s."
