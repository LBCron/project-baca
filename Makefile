-include .env
export

DB_USER ?= baca_user
DB_NAME ?= project_baca_db

CARGO_WATCH ?= $(shell which cargo-watch 2>/dev/null)
TRUNK ?= $(shell which trunk 2>/dev/null)

# Starts the development environment
dev:
	@echo "Project Baca — Development Environment"
	@echo "Run 'make dev-server' for backend (auto-reload on save)"
	@echo "Run 'make dev-web' for frontend WASM (Trunk hot-reload)"
	@echo "Run 'make db-up' to start database and services"

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
	@cd crates/web && trunk serve

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
	@echo "Running web test pyramid (requires db-up, dev-server :8080, dev-web :3000)..."
	@cargo test -p web --lib
	@cd crates/web/tests && ~/.venvs/webapp-testing/bin/python -m pytest e2e/ component/ integration/ --browser chromium -q

test-web-full:
	@echo "Web pyramid + visual + a11y + firefox smoke + strict-CSP shell check..."
	@python3 scripts/externalize_inline_scripts.py --check crates/web/dist
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
build:
	@echo "Building production release binaries..."
	@cargo build --workspace --release
	@echo "Building production WASM frontend bundle..."
	@cd crates/web && API_BASE_URL=$(API_BASE_URL) trunk build --release --public-url /

# Production compose stack (see .env.production.example). Dev stack stays
# on docker-compose.yml with host ports 5433/6380/9005/9006.
prod-build:
	@echo "Building production images..."
	@docker compose --env-file .env.production -f docker-compose.prod.yml build

prod-up:
	@echo "Starting production stack..."
	@docker compose --env-file .env.production -f docker-compose.prod.yml up -d

prod-down:
	@echo "Stopping production stack..."
	@docker compose --env-file .env.production -f docker-compose.prod.yml down

prod-logs:
	@docker compose --env-file .env.production -f docker-compose.prod.yml logs -f

# Purge test debris from shared dev services (stream PEL, draft rows,
# fake MinIO objects). Dev-only: never run against production data.
purge-test-debris:
	@echo "Purging test debris (pending PEL, probe drafts, PKfake objects, seed survivors)..."
	@set -a && . ./.env && set +a && cd python_worker && .venv/bin/python ../scripts/purge_test_debris.py

seed-dev:
	@echo "Seeding the development catalog (four CC0 works with fixed ids)..."
	@docker exec -i project_baca_db psql -U baca_user -d project_baca_db -v ON_ERROR_STOP=1 -q < scripts/seed_dev_catalog.sql
	@echo "Seeded. The web suite pins a0000000-0000-4000-8000-000000000001."

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -rf crates/web/dist

.PHONY: dev dev-server dev-web db-up db-down db-prune db-logs db-shell redis-shell migrate-up migrate-down migrate-status test test-unit test-smoke test-integration test-auth test-catalog test-semantic test-database test-performance test-performance-release test-load-stress test-api-boundary test-security test-reliability test-all test-live check build prod-build prod-up prod-down prod-logs clean purge-test-debris seed-dev worker-install worker-test worker-test-live worker worker-once test-admin test-domain test-web

