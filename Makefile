-include .env
export

DB_USER ?= baca_user
DB_NAME ?= project_baca_db

CARGO_WATCH ?= $(shell which cargo-watch 2>/dev/null)
TRUNK ?= $(shell which trunk 2>/dev/null)

# Dev controller: `make dev backend`, `make dev frontend`, `make dev stop`.
# It owns ports 8080 and 3000 across all checkouts and waits until each
# service answers (scripts/dev.sh); tests that need a server start it here.
DEV_SERVICES := backend frontend stop

dev:
ifeq ($(filter $(DEV_SERVICES),$(MAKECMDGOALS)),)
	@echo "Project Baca — Development Environment"
	@echo "make dev backend    (re)start the API on :8080 from this checkout, with its services"
	@echo "make dev frontend   serve the web on :3000 from this checkout (Trunk hot-reload)"
	@echo "make dev stop       stop both"
else
	@scripts/dev.sh $(filter $(DEV_SERVICES),$(MAKECMDGOALS))
endif

$(DEV_SERVICES):
	@$(if $(filter dev,$(MAKECMDGOALS)),:,echo "use: make dev $@" >&2; exit 2)

# Starts backend with live reload
dev-server:
	@echo "Starting backend development server..."
ifeq ($(strip $(CARGO_WATCH)),)
	@echo "cargo-watch not installed. Running standard 'cargo run -p server'."
	@cargo run -p server
else
	@echo "Live reload enabled via cargo-watch..."
	@cargo watch -q -c -w crates/server -w crates/infra -w crates/domain -w crates/shared -x "run -p server"
endif

# Starts frontend with Trunk native hot-reload
dev-web:
	@echo "Starting frontend development server (Trunk hot-reload)..."
	@cd crates/web && trunk serve --cargo-profile wasm-dev

# Serves the release bundle (wasm-release profile, wasm-opt) on :3000, for
# running the web suite against release optimisations; the `gallery` feature
# keeps the component story route the shipped bundle leaves out.
dev-web-release:
	@echo "Starting frontend on the release bundle (with the story gallery)..."
	@cd crates/web && trunk serve --release --cargo-profile wasm-release --features gallery

# Starts all infrastructure containers in detached mode
db-up:
	@echo "Starting database and infrastructure services..."
	@docker compose up -d

# Stops all infrastructure containers
db-down:
	@echo "Stopping infrastructure services..."
	@docker compose down

# Stop and remove all containers, networks, and volumes (WARNING: DB data will be lost)
db-prune:
	@echo "Removing all containers and volumes (WARNING: DB data will be lost)..."
	@docker compose down -v

# View container logs
db-logs:
	@docker compose logs -f

# Connect to the psql shell inside the 'postgres' container
db-shell:
	@docker compose exec postgres psql -U $(DB_USER) -d $(DB_NAME)

# Connect to the redis-cli shell inside the 'redis' container
redis-shell:
	@docker compose exec redis redis-cli

# Database migrations
migrate-up: 
	@echo "Applying database migrations (up)..."
	@./scripts/migrate.sh up

migrate-down:
	@echo "Rolling back last database migration (down)..."
	@./scripts/migrate.sh down

migrate-status:
	@echo "Checking database migration history..."
	@./scripts/migrate.sh status

# Failed, concurrent and repeated runs of the migration runner on a throwaway database.
test-migrate:
	@./scripts/test_migrate.sh

# Caddyfile.prod's client address in Docker: CF-Connecting-IP only from the
# connector's address, the peer otherwise (the host's bridge gateway
# included), no other address header upstream.
test-edge-ip:
	@./scripts/test_edge_client_ip.sh

# The request check of scripts/deploy.sh, the only input the deploy key sends.
test-deploy:
	@./scripts/test_deploy.sh

# Test suites
test: test-all

test-unit:
	@echo "Running unit tests across workspace libraries..."
	@cargo test --workspace --lib

test-smoke:
	@echo "Running smoke tests (boot, health, Swagger UI, infrastructure)..."
	@cargo test -p server --test it smoke_test::

test-integration:
	@echo "Running integration tests (routing, request-id, CORS, OpenAPI schemas)..."
	@cargo test -p server --test it integration_test::

test-auth:
	@echo "Running authentication and user integration tests..."
	@cargo test -p server --test it auth_test::

test-catalog:
	@echo "Running catalog, reader engine and gamification integration tests..."
	@cargo test -p server --test it catalog_test::

test-semantic:
	@echo "Running semantic AI, quote search and atomic cards integration tests..."
	@cargo test -p server --test it semantic_ai_test::

test-performance:
	@echo "Running performance and SLA benchmark tests..."
	@cargo test -p server --test performance_test

test-performance-release:
	@echo "Running SLA benchmarks on release build (production claim)..."
	@cargo test --release -p server --test performance_test

test-reliability:
	@echo "Running reliability and fault injection tests..."
	@cargo test -p server --test it reliability_test::

test-database:
	@echo "Running database integrity tests (constraints, cascades, rollbacks, query plans)..."
	@cargo test -p server --test it database_test::

test-load-stress:
	@echo "Running load and stress tests (200 concurrent tasks, rate limit saturation)..."
	@cargo test -p server --test load_stress_test

test-api-boundary:
	@echo "Running API edge-case and boundary tests (404/405, malformed payloads, pagination)..."
	@cargo test -p server --test it api_boundary_test::

test-security:
	@echo "Running OWASP security vulnerability tests..."
	@cargo test -p server --test it security_owasp_test::

test-admin:
	@echo "Running admin ingestion, job monitoring and analytics tests..."
	@cargo test -p server --test it admin_test::

test-domain:
	@echo "Running domain-core pure unit tests (no database)..."
	@cargo test -p domain

test-web:
	@echo "Running web test pyramid (requires make dev backend frontend)..."
	@cargo test -p web --lib
	@cd crates/web/tests && ~/.venvs/webapp-testing/bin/python -m pytest e2e/ component/ integration/ --browser chromium -q

test-web-full:
	@echo "Release bundle budget + strict-CSP shell, then web pyramid + visual + a11y + firefox smoke..."
	@$(MAKE) --no-print-directory build-web WEB_DIST=$(CURDIR)/target/web-release
	@python3 scripts/externalize_inline_scripts.py --check target/web-release
	@cd crates/web/tests && ~/.venvs/webapp-testing/bin/python -m pytest . --browser chromium -q
	@cd crates/web/tests && ~/.venvs/webapp-testing/bin/python -m pytest e2e/test_smoke.py --browser chromium --browser firefox -q

test-all:
	@echo "Running complete test suite..."
	@cargo test --workspace

test-live:
	@echo "Running live-services tests (needs db-up)..."
	@BACA_LIVE_TEST=1 cargo test -p server -- --ignored --test-threads=1

# Check compilation across all crates (backend & WASM frontend)
check:
	@echo "Checking backend and shared crates..."
	@cargo check --workspace
	@echo "Checking WASM frontend crate..."
	@cargo check -p web --target wasm32-unknown-unknown
	@echo "Running clippy lints (deny warnings)..."
	@cargo clippy --workspace --all-targets -- -D warnings
	@echo "Checking formatting..."
	@cargo fmt --check

# Supply chain: advisories, bans, sources, and licences of both Cargo
# workspaces (deny.toml), then the worker's Python dependencies.
audit:
	@cargo deny check advisories bans sources licenses
	@cargo deny --manifest-path apps/tauri/Cargo.toml --config deny.toml check advisories bans sources licenses
	@uvx pip-audit -r python_worker/requirements.txt

# Ingestion worker
worker-install:
	@echo "Creating worker venv and installing dependencies..."
	@cd python_worker && uv venv .venv && uv pip install -r requirements.txt

worker-test:
	@echo "Running worker unit tests (no services required)..."
	@cd python_worker && .venv/bin/python -m unittest test_worker

worker-test-live:
	@echo "Running worker live tests (needs Redis + Postgres)..."
	@cd python_worker && RECLAIM_IDLE_MS=0 .venv/bin/python -m unittest test_worker_live -v

backfill-paragraphs:
	@echo "Indexing the paragraphs of every stored chapter and anchoring its chunks (one-off, idempotent)..."
	@set -a && . ./.env && set +a && cd python_worker && .venv/bin/python worker.py --backfill-paragraphs

worker:
	@echo "Starting ingestion worker (long-running consumer)..."
	@cd python_worker && .venv/bin/python worker.py

worker-once:
	@echo "Processing a single ingestion job then exiting..."
	@cd python_worker && .venv/bin/python worker.py --once

# Production release build
# API base baked into the WASM bundle at compile time (Trunk has no runtime
# env); override per environment, e.g. API_BASE_URL=https://example.com/api/v1
API_BASE_URL ?= http://localhost:8080/api/v1
WEB_DIST ?= $(CURDIR)/crates/web/dist
build:
	@echo "Building production release binaries..."
	@cargo build --workspace --release
	@$(MAKE) --no-print-directory build-web

# Release WebAssembly bundle: wasm-release profile, wasm-opt -Oz (Trunk
# fetches binaryen when it is not installed), then the size budget.
build-web:
	@echo "Building the release WASM bundle into $(WEB_DIST)..."
	@cd crates/web && API_BASE_URL=$(API_BASE_URL) trunk build --release --cargo-profile wasm-release --public-url / --dist $(WEB_DIST)
	@python3 scripts/check_bundle_budget.py $(WEB_DIST)

# The bundle the edge image serves: the release build plus `.br` and `.gz`
# copies Caddy sends as they are. Kept apart from build-web, whose output the
# Tauri app embeds whole and would carry the copies unused.
build-web-edge: build-web
	@python3 scripts/precompress.py $(WEB_DIST)

# Tauri shell (apps/tauri, `cargo install tauri-cli --version "^2" --locked`).
# The app injects APP_API_BASE as window.__ROTARIA_API__; desktop needs
# webkit2gtk-4.1, Android needs ANDROID_HOME, NDK_HOME, and JAVA_HOME, and an
# emulator reaches the host API at 10.0.2.2.
APP_API_BASE ?= http://localhost:8080/api/v1
APP_ANDROID_API_BASE ?= http://10.0.2.2:8080/api/v1
APP_ANDROID_TARGET ?= x86_64
app-dev:
	@cd apps/tauri && ROTARIA_API_BASE=$(APP_API_BASE) cargo tauri dev

app-build:
	@cd apps/tauri && ROTARIA_API_BASE=$(APP_API_BASE) cargo tauri build

app-test:
	@cd apps/tauri && cargo test

app-android-dev:
	@cd apps/tauri && ROTARIA_API_BASE=$(APP_ANDROID_API_BASE) cargo tauri android dev

app-android-build:
	@cd apps/tauri && ROTARIA_API_BASE=$(APP_ANDROID_API_BASE) cargo tauri android build --apk --debug --target $(APP_ANDROID_TARGET)

# Production compose stack (see .env.production.example). Dev stack stays
# on docker-compose.yml with host ports 5433/6380/9005/9006. A laptop run takes
# PROD_ENV=<a copy without the real CLOUDFLARE_TUNNEL_TOKEN>: a second connector
# on the production tunnel would take a share of rotaria.site's traffic.
PROD_ENV ?= .env.production
# .env.deploy, written by each deploy on the host, pins the release images by digest.
PROD_COMPOSE = docker compose --env-file $(PROD_ENV) $(if $(wildcard .env.deploy),--env-file .env.deploy) -f docker-compose.prod.yml
prod-build:
	@echo "Building production images..."
	@$(PROD_COMPOSE) build

# Server, worker, edge, and backup images of ROTARIA_VERSION from GHCR (public, no login).
prod-pull:
	@$(PROD_COMPOSE) pull

prod-up:
	@echo "Starting production stack..."
	@$(PROD_COMPOSE) up -d

prod-down:
	@echo "Stopping production stack..."
	@$(PROD_COMPOSE) down

# Pending migrations against the production database, each in its own
# transaction under an advisory lock; safe to rerun and to run twice at once.
prod-migrate:
	@$(PROD_COMPOSE) --profile ops run --rm migrate up

# rotaria_app, the role server and worker use, on a database volume that
# existed before it (a new volume gets it at init); rerun after changing
# POSTGRES_APP_PASSWORD. `up` first recreates postgres when its environment
# or mounts changed, so the script runs with the current password.
prod-db-role:
	@$(PROD_COMPOSE) up -d --wait postgres
	@$(PROD_COMPOSE) exec -T postgres /docker-entrypoint-initdb.d/10-app-role.sh

prod-migrate-status:
	@$(PROD_COMPOSE) --profile ops run --rm migrate status

# One encrypted dump to the backup bucket now, as the backup service does every
# 6 hours; a deploy runs it before migrating (GUIDE section 8).
prod-backup:
	@$(PROD_COMPOSE) run --rm backup pg_backup.sh

# Encrypted copy of PROD_ENV in the backup bucket (env/), after every change of it.
prod-env-copy:
	@test -s "$(PROD_ENV)" || { echo "$(PROD_ENV) is missing or empty" >&2; exit 2; }
	@$(PROD_COMPOSE) run --rm --no-deps -T backup pg_backup.sh --env-copy < $(PROD_ENV)

# Restores the newest dump (or OBJECT=pg/YYYY/MM/DD/<name>) into a throwaway
# Postgres and prints its row counts. AGE_IDENTITY is the owner's age private
# key file; it is read through a read-only mount and never copied.
prod-restore-drill:
	@test -n "$(AGE_IDENTITY)" || { echo "Usage: make prod-restore-drill AGE_IDENTITY=<age key file> [OBJECT=pg/YYYY/MM/DD/<name>]" >&2; exit 2; }
	@PROD_COMPOSE="$(PROD_COMPOSE)" scripts/pg_restore_drill.sh

prod-logs:
	@$(PROD_COMPOSE) logs -f

# psql on the production database, as the app role by default so a slip cannot
# change the schema; PROD_PSQL_USER=baca_user for the schema owner.
PROD_PSQL_USER ?= rotaria_app
prod-psql:
	@$(PROD_COMPOSE) exec postgres psql -U $(PROD_PSQL_USER) -d project_baca_db

# Admin role for one active account, through the app role. The address reaches
# psql through the environment, never the command line or the SQL text.
prod-grant-admin:
	@test -n "$$EMAIL" || { echo "Usage: make prod-grant-admin EMAIL=<address of an active account>" >&2; exit 2; }
	@row=$$(printf '%s\n' '\getenv email EMAIL' \
		"UPDATE users AS u SET role = 'admin', updated_at = now() FROM (SELECT id, role FROM users WHERE email = lower(btrim(:'email')) AND is_active FOR UPDATE) AS old WHERE u.id = old.id RETURNING u.id, old.role, u.role;" \
		| $(PROD_COMPOSE) exec -T -e EMAIL postgres psql -X -A -t -q -F ' ' -v ON_ERROR_STOP=1 -U rotaria_app -d project_baca_db) || exit 1; \
	set -- $$row; \
	test -n "$$1" || { echo "No active account has that email; nothing changed." >&2; exit 1; }; \
	echo "User $$1: role $$2 -> $$3."; \
	echo "The role applies after the user's next sign-in or token refresh (up to 60 minutes)."

# After migrating and before the new server takes traffic: until it has run,
# in-book search finds nothing in books ingested earlier.
prod-backfill-paragraphs:
	@$(PROD_COMPOSE) run --rm worker python worker.py --backfill-paragraphs

# Hardens the production host over SSH (GUIDE section 8). HOST is the admin
# login with passwordless sudo; CHECK=1 only verifies; DEPLOY_PUBKEY installs
# the deploy key behind its forced command. scripts/deploy.sh travels on stdin
# ahead of the script and becomes that command.
prod-provision:
	@test -n "$(HOST)" || { echo "Usage: make prod-provision HOST=<admin>@<host> [CHECK=1] [DEPLOY_PUBKEY='ssh-ed25519 ...']"; exit 2; }
	@{ printf 'DEPLOY_SCRIPT_B64=%s\n' "$$(base64 -w0 scripts/deploy.sh)"; cat scripts/provision_host.sh; } \
		| ssh $(HOST) "sudo -n env DEPLOY_PUBKEY='$(DEPLOY_PUBKEY)' bash -s -- $(if $(CHECK),--check)"

# Requests URL at RATE per second for DURATION seconds and sums up statuses,
# connection errors, and latency (scripts/deploy_probe.sh); run it during a
# deploy. CF_ACCESS_CLIENT_ID and CF_ACCESS_CLIENT_SECRET add the Access token.
deploy-probe:
	@scripts/deploy_probe.sh "$$URL" "$$RATE" "$$DURATION"

# Purge test debris from shared dev services (stream PEL, draft rows,
# fake MinIO objects). Dev-only: never run against production data.
purge-test-debris:
	@echo "Purging test debris (pending PEL, probe drafts, PKfake objects, seed survivors)..."
	@set -a && . ./.env && set +a && cd python_worker && .venv/bin/python ../scripts/purge_test_debris.py

seed-dev:
	@echo "Seeding the development catalog (four CC0 works with fixed ids)..."
	@docker exec -i project_baca_db psql -U baca_user -d project_baca_db -v ON_ERROR_STOP=1 -q < scripts/seed_dev_catalog.sql
	@$(MAKE) --no-print-directory backfill-paragraphs
	@echo "Seeded. The web suite pins a0000000-0000-4000-8000-000000000001."

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -rf crates/web/dist

.PHONY: audit dev $(DEV_SERVICES) dev-server dev-web dev-web-release build-web build-web-edge app-dev app-build app-test app-android-dev app-android-build db-up db-down db-prune db-logs db-shell redis-shell migrate-up migrate-down migrate-status test-migrate test-edge-ip test-deploy test test-unit test-smoke test-integration test-auth test-catalog test-semantic test-database test-performance test-performance-release test-load-stress test-api-boundary test-security test-reliability test-all test-live check build prod-build prod-pull prod-up prod-migrate prod-db-role prod-migrate-status prod-backup prod-env-copy prod-restore-drill prod-down prod-logs prod-psql prod-grant-admin prod-backfill-paragraphs prod-provision deploy-probe clean purge-test-debris backfill-paragraphs seed-dev worker-install worker-test worker-test-live worker worker-once test-admin test-domain test-web

