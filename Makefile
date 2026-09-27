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
	@cargo test -p server --test smoke_test

test-integration:
	@echo "Running integration tests (routing, request-id, CORS, OpenAPI schemas)..."
	@cargo test -p server --test integration_test

test-auth:
	@echo "Running authentication and user integration tests..."
	@cargo test -p server --test auth_test

test-catalog:
	@echo "Running catalog, reader engine and gamification integration tests..."
	@cargo test -p server --test catalog_test

test-semantic:
	@echo "Running semantic AI, quote search and atomic cards integration tests..."
	@cargo test -p server --test semantic_ai_test

test-performance:
	@echo "Running performance and SLA benchmark tests..."
	@cargo test -p server --test performance_test

test-reliability:
	@echo "Running reliability and fault injection tests..."
	@cargo test -p server --test reliability_test

test-database:
	@echo "Running database integrity tests (constraints, cascades, rollbacks, query plans)..."
	@cargo test -p server --test database_test

test-load-stress:
	@echo "Running load and stress tests (200 concurrent tasks, rate limit saturation)..."
	@cargo test -p server --test load_stress_test

test-api-boundary:
	@echo "Running API edge-case and boundary tests (404/405, malformed payloads, pagination)..."
	@cargo test -p server --test api_boundary_test

test-security:
	@echo "Running OWASP security vulnerability tests..."
	@cargo test -p server --test security_owasp_test

test-all:
	@echo "Running complete test suite..."
	@cargo test --workspace

# Check compilation across all crates (backend & WASM frontend)
check:
	@echo "Checking backend and shared crates..."
	@cargo check --workspace
	@echo "Checking WASM frontend crate..."
	@cargo check -p web --target wasm32-unknown-unknown

# Production release build
build:
	@echo "Building production release binaries..."
	@cargo build --workspace --release
	@echo "Building production WASM frontend bundle..."
	@cd crates/web && trunk build --release

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -rf crates/web/dist

.PHONY: dev dev-server dev-web db-up db-down db-prune db-logs db-shell redis-shell migrate-up migrate-down migrate-status test test-unit test-smoke test-integration test-auth test-catalog test-semantic test-database test-performance test-load-stress test-api-boundary test-security test-reliability test-all check build clean

