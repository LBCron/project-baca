#!/usr/bin/env bash
set -euo pipefail

# Creates or updates rotaria_app, the role server and worker connect as: it
# may read and write the rows of the schema but not change it, own nothing,
# and is no superuser, so a code-execution bug cannot reach COPY ... TO
# PROGRAM or the migrations. POSTGRES_USER owns the schema and runs them.
#
# Runs from /docker-entrypoint-initdb.d on an empty volume, and through
# `make prod-db-role` on an existing one; safe to rerun, and a rerun applies
# a changed POSTGRES_APP_PASSWORD.

: "${POSTGRES_USER:?}" "${POSTGRES_DB:?}" "${POSTGRES_APP_PASSWORD:?set POSTGRES_APP_PASSWORD}"

psql -v ON_ERROR_STOP=1 -X -q --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" \
    -v owner="$POSTGRES_USER" -v db="$POSTGRES_DB" -v app_password="$POSTGRES_APP_PASSWORD" <<'SQL'
SELECT 'CREATE ROLE rotaria_app' WHERE NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'rotaria_app') \gexec
ALTER ROLE rotaria_app WITH LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
    PASSWORD :'app_password';
GRANT CONNECT ON DATABASE :"db" TO rotaria_app;
GRANT USAGE ON SCHEMA public TO rotaria_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO rotaria_app;
GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA public TO rotaria_app;
-- Tables and sequences the owner's later migrations create.
ALTER DEFAULT PRIVILEGES FOR ROLE :"owner" IN SCHEMA public
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO rotaria_app;
ALTER DEFAULT PRIVILEGES FOR ROLE :"owner" IN SCHEMA public
    GRANT USAGE, SELECT, UPDATE ON SEQUENCES TO rotaria_app;
SQL
echo "rotaria_app ready on ${POSTGRES_DB}."
