---
type: Developer Guide
title: Developer Guide
tags:
  - developer
  - guide
  - okf
  - onboarding
  - setup
  - workflow
---

# Developer Guide

Build, run, and test Project Baca locally. Deployment is in [ARCHITECTURE.md](ARCHITECTURE.md) section 8.

## 1. Prerequisites

- Rust stable with `rustup target add wasm32-unknown-unknown` and `rustup component add clippy rustfmt`.
- Trunk (`cargo install trunk`). `cargo-watch` is optional; `make dev-server` falls back to `cargo run`.
- Docker Compose v2.
- Python 3.11+ and `uv` (worker and janitor).
- Web suite: a venv at `~/.venvs/webapp-testing` from `crates/web/tests/requirements-test.txt`, with Playwright Chromium and Firefox installed.
- App shell (optional): `cargo install tauri-cli --version "^2" --locked`; desktop needs webkit2gtk-4.1; Android needs the SDK with `ANDROID_HOME`, `NDK_HOME`, and `JAVA_HOME` set.

## 2. Services (`make db-up`)

| Service | Image | Host port (loopback only) |
|---|---|---|
| PostgreSQL 17 + pgvector | `pgvector/pgvector:pg17` | 5433 |
| Redis 7 | `redis:7-alpine` | 6380 |
| MinIO | `cgr.dev/chainguard/minio` | 9005 (S3 API), 9006 (console) |
| Mailpit | `axllent/mailpit` | 1025 (SMTP), 8025 (web UI) |

Credentials and bucket names are in `.env.example`. At boot the server checks both buckets with a one-key listing and creates one only when it is missing.

## 3. Configuration (`.env`)

| Variable | Default | Meaning |
|---|---|---|
| `APP_ENV`, `APP_HOST`, `APP_PORT` | `development`, `0.0.0.0`, `8080` | `production` refuses the default JWT secret and unmounts Swagger; `.env.example` sets `APP_HOST=127.0.0.1` so the dev API stays off the network (the Android emulator reaches it at 10.0.2.2), while the server image keeps `0.0.0.0` |
| `CORS_ALLOWED_ORIGINS` | localhost:3000 origins, `tauri://localhost`, `http://tauri.localhost` | comma-separated allowlist of exact origins; the last two are the Tauri app WebView origins. `production` refuses `*`, `null`, a path or trailing slash, and plain `http://` other than `http://tauri.localhost`; the production compose sets `https://${DOMAIN}` and the two Tauri origins |
| `LOG_FORMAT` | `text` | `json` for NDJSON logs |
| `TRUST_PROXY_HEADERS`, `TRUSTED_IP_HEADER` | `false`, `x-forwarded-for` | with trust on, the client address is read from that one header only (`cf-connecting-ip` behind Cloudflare) |
| `PUBLIC_RATE_LIMIT_PER_MINUTE` | `0` (off) | per-client cap on unauthenticated reads; production compose sets 600 |
| `BOOK_SEARCH_RATE_LIMIT_PER_MINUTE` | `0` (off) | per-client cap on in-book search; production compose sets 30 |
| `ARGON2_MAX_CONCURRENCY` | cores, at most 8 | Argon2 hashes and verifies run at once (19 MiB each); a login or signup that waits 2 s for a slot answers 503; 0 is refused |
| `OTP_HOURLY_BUDGET`, `AI_DAILY_BUDGET` | `0` (off) | ceilings across every client: signup mail per hour (past it signup answers 503 and logs `otp_budget_exhausted` once) and semantic searches per day (past it 429 `AI_RATE_LIMITED` and `ai_budget_exhausted`; guests share a fifth); a value that is not a whole number stops the boot; production compose sets 100 and 5000, and production refuses to boot with either at 0 or without `TRUST_PROXY_HEADERS=true` |
| `JWT_SECRET`, `JWT_ACCESS_EXPIRY_MINUTES`, `JWT_REFRESH_EXPIRY_DAYS` | dev secret, `1440`, `14` | production compose pins access tokens to 60 minutes |
| `SESSION_MAX_DAYS` | `90` | absolute session lifetime from sign-in; refresh past it answers 401 and the reader signs in again |
| `DATABASE_URL`, `DATABASE_MAX_CONNECTIONS`, `DATABASE_MIN_CONNECTIONS`, `DATABASE_CONNECT_TIMEOUT_SECS`, `DATABASE_IDLE_TIMEOUT_SECS`, `DATABASE_STATEMENT_TIMEOUT_MS` | dev URL, `20`, `5`, `10`, `300`, `5000` | SeaORM pool; Postgres cancels an API statement past the timeout |
| `REDIS_URL` | `redis://localhost:6380/0` | OTP, sessions, rate limits, streams |
| `S3_ENDPOINT`, `S3_REGION`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY`, `S3_BUCKET_EPUBS`, `S3_BUCKET_COVERS`, `S3_FORCE_PATH_STYLE` | MinIO dev values | any S3-compatible store |
| `SMTP_HOST`, `SMTP_PORT`, `SMTP_SECURITY`, `SMTP_USER`, `SMTP_PASSWORD`, `SMTP_FROM_EMAIL`, `SMTP_FROM_NAME` | Mailpit, `starttls` | `SMTP_SECURITY=none` for Mailpit (refused in production); credentials sent only when `SMTP_USER` is set; signup answers 502 when the relay is unreachable |
| `EMBEDDING_PROVIDER`, `GEMINI_API_KEY`, `EMBEDDING_MODEL_NAME`, `EMBEDDING_DIMENSION` | `gemini`, none, `gemini-embedding-2`, `768` | shared by server (query embeddings) and worker; `AI_*` aliases accepted |
| `LLM_MODEL_NAME`, `RECLAIM_IDLE_MS` | `gemini-flash-latest`, `300000` | worker only |
| `API_BASE_URL` | `http://localhost:8080/api/v1` | baked into the WASM bundle at build time (`make build API_BASE_URL=...`) |

## 4. Run

```bash
make migrate-up seed-dev
make dev backend       # (re)start the API on :8080 from this checkout, wait until healthy
make dev frontend      # trunk serve on 127.0.0.1:3000 from this checkout, cargo profile wasm-dev (make dev-web-release for the shipped bundle)
make dev stop          # stop both
make worker            # after make worker-install
```

- `make dev` is the controller for tests and live checks: ports 8080 and 3000 are shared by every checkout and worktree, so it records which checkout runs each, takes a port over from another checkout of this repo (never from a foreign process), links the main checkout's `.env` into a new worktree, shares `$MAIN/target`, and logs to `~/.local/state/rotaria-dev/`. `make dev-server` (cargo-watch) and `make dev-web` stay for hands-on editing.
- Swagger UI `/swagger-ui`, OpenAPI `/api-docs/openapi.json` outside production.
- Every response carries a server-minted `x-request-id`; client values are ignored.
- Migrations: `make migrate-up`, `make migrate-down`, `make migrate-status` over paired SQL files in `migrations/`. Each file and its `schema_migrations` row run in one transaction under an advisory lock, so a failed file leaves nothing and two runs at once apply each file once (`make test-migrate`). With `MIGRATE_DATABASE_URL` set, `scripts/migrate.sh` talks to that database with a local `psql` instead of the dev compose service.
- Production: `make prod-migrate` (and `make prod-migrate-status`) runs the `migrate` service of the `ops` profile, which mounts `migrations/` and the runner of the checked-out tag read-only; check out the release tag first. Run it before the new server takes traffic.
- Expand/contract: every migration must keep working with the server release before it (add a column nullable or with a default, backfill, and drop the old one only in a later release). Roll back by rolling forward with a new migration; the pre-migration dump is the emergency path. A statement that cannot run inside a transaction (`CREATE INDEX CONCURRENTLY`, which fails under `psql -1`) goes in its own file outside the runner, applied by hand with `psql -f` and recorded with `INSERT INTO schema_migrations`; none exists yet.
- Paragraph index and quote anchors for books ingested before them: `make backfill-paragraphs` (production `make prod-backfill-paragraphs`), one book per transaction, idempotent, no embeddings paid. Run it after migrating and before the new server takes traffic, or in-book search finds nothing in those books.
- Dev data: `make seed-dev` installs four CC0 books with fixed ids (the web suite pins `a0000000-0000-4000-8000-000000000001`). `make purge-test-debris` removes rows, stream entries, and objects left by crashed test runs. Never run either against production.

App shell:

```bash
make app-test                         # deep-link routing unit tests
make app-dev                          # desktop window on the dev API
make app-android-build                # debug APK (x86_64 emulator) talking to 10.0.2.2:8080
adb install -r apps/tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk
adb shell am start -a android.intent.action.VIEW -d rotaria://book/<book uuid> io.github.aprxty3.rotaria
```

## 5. Tests

| Target | Scope | Needs |
|---|---|---|
| `make check` | `cargo check` native and wasm, clippy with `-D warnings`, `fmt --check` | nothing |
| `make audit` | `cargo deny` (advisories, bans, sources, licences per `deny.toml`) on both Cargo workspaces, `pip-audit` on the worker | `cargo-deny`, `uv` |
| `make test-domain` | 16 pure domain tests | nothing |
| `cargo test -p infra`, `cargo test -p server --lib` | config, repositories, helpers (31 + 6) | nothing |
| `make test-<module>` | one module of `crates/server/tests/it`: smoke, integration, auth, catalog, semantic, admin, database, api-boundary, security, reliability | `db-up` |
| `cargo test -p server --test it` | all functional modules, 109 tests (9 live tests ignored) | `db-up` |
| `make test-live` | the 9 ignored live tests, single-threaded | `db-up` |
| `make test-performance`, `make test-performance-release` | 4 p95 latency benchmarks | `db-up`, idle machine |
| `make test-load-stress` | 200 concurrent tasks and rate-limit saturation | `db-up` |
| `make test-edge-ip` | `Caddyfile.prod` in Docker: only the connector's address names the client through `CF-Connecting-IP`; any other peer (another container on the tunnel subnet, the host through the bridge gateway, another private subnet) is the client, no other address header reaches the API | Docker |
| `make worker-test`, `make worker-test-live` | 27 worker unit tests; dead letters, claim refresh, and SIGTERM release on probe streams | nothing; `db-up` |
| `make test-web` | `cargo test -p web --lib` plus Playwright e2e, component, integration | `db-up`, API on :8080, web on :3000, Mailpit |
| `make test-web-full` | builds the release bundle into `target/web-release` (size budget, no inline script), then adds visual goldens, axe a11y, and Firefox smoke | same; serve the release bundle with `make dev-web-release` |
| `make test-all` | `cargo test --workspace` (includes performance and load) | `db-up` |

When CI runs the heavy jobs (`check`, `backend`, `web`; decided by `scripts/ci_scope.py`, whose `--self-test` runs first in the `changes` job):

| Run | Heavy jobs |
|---|---|
| Pull request to `main`, push to `main`, manual run | all |
| Pull request to `develop`, direct push to `develop` | `check` and `backend` unless only `*.md` changed; `web` only when `crates/web`, `crates/shared`, `Cargo.toml`, `Cargo.lock`, `.github/`, `Makefile`, or `scripts/trunk.sha256` changed |
| Push to `develop` of a pull request's merge commit | none (the pull request already ran them) |
| Nightly 02:00 WIB on `develop` | all, when `develop` moved in the last 25 hours |

`zizmor`, `audit`, and `changes` run every time. Skipped jobs report as skipped, which satisfies required checks.

Notes:

- Functional tests build the Axum app in-process against the shared dev services and delete every row they create (`TestHarness::track_*`, `harness.cleanup()`), so row counts match before and after a run.
- The web suite runs from `crates/web/tests` with `--browser chromium`. Contexts pin `color_scheme=dark`; the a11y suite also scans the Paper theme. Goldens are 1280x720 under `tests/visual/test_visual.py-snapshots/`; delete one to regenerate it after an intended change and review the image before committing.
- Benchmarks follow machine load. Rerun on an idle machine before reading a failure as a regression.
- Do not run the web suite while heavy backend suites run; Playwright `networkidle` waits can time out.
- Signup is capped at 10 per hour per client (an IPv6 client counts as its /64). Delete the Redis key `otp:signup:ip:127.0.0.1` if local auth tests answer 429.

## 6. Troubleshooting

- Port collisions: the dev stack uses 5433, 6380, 9005, 9006, 1025, 8025.
- OTP mail never arrives: set `SMTP_SECURITY=none`; signup answers 502 while SMTP is unreachable.
- Uploads fail with 500: object storage is unreachable. In development the server boots without it; in production it refuses to boot and logs `Object storage unavailable in production` with the bucket and HTTP status (403: wrong key or a token without access to that bucket; a missing bucket is created only when the key may create buckets, which an R2 Object Read and Write token may not).
- Worker never consumes: check `XINFO GROUPS stream:epub_ingestion`; the group is created when the worker starts.
- Frontend change not visible: Trunk watches `src`, `style`, `index.html`, `sw.js`, `boot.js`, the manifest, `icons`, and `fonts` of `crates/web`; a `crates/shared` change needs a Trunk restart.

## 7. Reader controls

- Turn a page: drag anywhere on a touch screen, or from the outer 22% of the page with a mouse. Release past the middle or flick to finish; release early to fall back. Tap the edges, use the edge arrows on wide screens, or press Space / Shift+Space, the arrow keys, PageUp / PageDown. Home and End jump within the chapter.
- Center tap dims the bars; tap again to restore them. Two fingers zoom the page. With a button focused, Space presses the button instead of turning. The chapter title in the bar opens the chapter drawer; the search icon searches the book (the saved copy when offline). Selecting text offers "Save quote". The first time a chapter ends, a closing card offers its insight cards or the next chapter. The last page of the book resists, and a forward tap opens the finale card.
- Reduced motion: a short crossfade replaces the paper animation.

## 8. CI and releases

Branches: work lands on `develop`; `main` takes what is ready to release.

| Workflow | Trigger | Runs |
|---|---|---|
| `.github/workflows/ci.yml` | push to `develop` or `main`, pull requests, nightly, manual (heavy jobs per section 5) | `zizmor`: workflow audit, red on a medium or high finding. `audit`: `make audit`. `check`: `make check` and the version check. `backend`: the dev compose stack (`.github/actions/dev-stack`), domain, infra, and server unit tests, the functional suites, live auth tests, load-stress, worker unit and live tests. `web`: `make build-web` (size budget, no inline script), then the API and the release bundle served by Trunk, `make test-web` and the axe suite on Chromium |
| `.github/workflows/codeql.yml` | pull requests to `main` or `develop`, push to `main`, Mondays, manual | CodeQL for Actions, JavaScript, Python, and Rust (`build-mode: none`); the generated Kotlin of the Tauri Android shell is left out because it cannot be built on the runner |
| `.github/workflows/release.yml` | tag `vX.Y.Z` | the tag must equal the workspace version; images `ghcr.io/aprxty3/project-baca/{server,worker,edge}` for `linux/amd64` and `linux/arm64` (each built natively, arm64 on `ubuntu-24.04-arm`, then joined into one multi-arch tag) tagged `X.Y.Z` and `X.Y`, each with an SBOM, provenance, and a GitHub attestation; a GitHub release with the CHANGELOG section as notes (pre-release while the major version is 0) |

Production host: one VM, VPS-2 (ADR-46), runs the whole stack from `docker-compose.prod.yml` (ARCHITECTURE section 8); `make prod-build prod-migrate prod-up PROD_ENV=<file>` runs the same stack on a laptop before a deploy, from a copy of `.env.production.example` whose `CLOUDFLARE_TUNNEL_TOKEN` is not the real one (its `cloudflared` restarts in a loop and the laptop never joins the production tunnel). It holds a shallow clone of the deployed tag in `/srv/rotaria/repo` (`git clone --depth 1 --branch vX.Y.Z https://github.com/aprxty3/project-baca.git /srv/rotaria/repo`) with the compose file, `migrations/`, and `.env.production` with `ROTARIA_VERSION=X.Y.Z`. Deploy: check out the new tag (`git fetch --depth 1 origin tag vX.Y.Z && git checkout vX.Y.Z`), set `ROTARIA_VERSION`, then `make prod-pull prod-migrate prod-up`. A database volume created before the `rotaria_app` role existed takes `make prod-db-role` once before `prod-up` (and again after changing `POSTGRES_APP_PASSWORD`); a new volume gets the role when Postgres initialises it. The edge trusts the client address only from `cloudflared` at `10.203.53.2`, so after `prod-up` `docker network inspect <project>_tunnel -f '{{range .IPAM.Config}}{{.Subnet}}{{end}}'` must print `10.203.53.0/29` (`<project>` is the compose project, the clone's directory name by default); if a `tunnel` network from an earlier release shows another subnet, `make prod-down prod-up` recreates it. Roll back by setting the previous tag and version and running `make prod-pull prod-up`; migrations are expand/contract, so the older server runs on the newer schema. The repository and the GHCR packages are public, so clone and pull need no credentials. When the repository turns private before the beta, the host gets a read-only deploy key for the clone and `docker login ghcr.io` with a `read:packages` token used only by that host, and the packages are republished as private. `make prod-build` builds the same images from the checkout instead, tagged with `ROTARIA_VERSION`.

Operations on the host, from `/srv/rotaria/repo`: `make prod-psql` opens psql as `rotaria_app`, which cannot change the schema (`PROD_PSQL_USER=baca_user` for the owner). `make prod-grant-admin EMAIL=<address>` gives one active account the admin role through `rotaria_app` and prints its id with the old and new role; an unknown or inactive address exits 1 and changes nothing. The new role reaches the account's access token at its next sign-in or refresh, so up to 60 minutes later.

Host hardening: `make prod-provision HOST=ubuntu@<vps2>` pipes `scripts/provision_host.sh` to the host and runs it as root through passwordless `sudo` from the admin account; it refuses to start when that account has no SSH key. Each run converges, then verifies and exits 1 on a failed check; a second run prints `no change`. `CHECK=1` only verifies. It manages: the drop-in `/etc/ssh/sshd_config.d/00-rotaria.conf` (keys only, no root login, validated with `sshd -t` before a reload that keeps open sessions); the ufw rules below, adding SSH and Beszel before it deletes any rule that allows 80, 443, 5432, or 6379 (a rule that mixes those with a kept port is left for a person and fails the check); `/etc/apt/apt.conf.d/99rotaria-unattended-upgrades`; time sync (chrony, the 26.04 default); a 4 GB `/swapfile` when the host has no swap; Docker from `download.docker.com` when Docker is missing; the `deploy` user; `/srv/rotaria`. It leaves alone every other ufw rule, `/etc/ufw/before.rules` (Oracle's instance-services rules), `/etc/ufw/after.rules` (the ufw-docker `DOCKER-USER` block), and a Docker that is already installed.

| Inbound to VPS-2 | From | Why |
|---|---|---|
| `22/tcp` | internet | SSH with keys only, no root login (ADR-47); deploys from GitHub Actions |
| `45876/tcp` | `10.0.0.100` only | Beszel hub on VPS-1 reaching the agent |
| anything else | - | denied; the site is reached only through the outbound Cloudflare Tunnel |

Ports published by Docker bypass ufw, and the ufw-docker block admits `10.0.0.0/8` in `DOCKER-USER`, so the production compose publishes no port; the check fails when a container publishes 5432 or 6379.

VCN security list (OCI Console, Networking, the VCN, Security Lists; stateful, egress open for the Tunnel, apt, GHCR, and R2):

| Ingress | Source | Why |
|---|---|---|
| TCP 22 | `0.0.0.0/0` | SSH (ADR-47) |
| TCP 45876 | `10.0.0.100/32` | Beszel |
| ICMP type 3 code 4 | `0.0.0.0/0` | path MTU discovery (Oracle default) |
| ICMP type 3 | VCN CIDR | Oracle default |

No ingress for 80, 443, 5432, or 6379.

Fallback when SSH is lost: OCI Console, the instance, Console connection (serial console through Cloud Shell). The serial login asks for a local password, so the admin account keeps one in the owner's password manager (`sudo passwd ubuntu`); SSH still refuses passwords. Without it, reboot from the console and edit the GRUB entry to boot a root shell.

Checks after a run: `make prod-provision HOST=ubuntu@<vps2> CHECK=1`; from outside, `nmap -Pn <public IP>` finds only 22; from VPS-1, `nc -zv -w 3 10.0.0.44 5432` and `6379` fail; Beszel still shows VPS-2.

Reboot window: unattended-upgrades installs Ubuntu security updates daily and, when one needs it, reboots at 04:00 WIB, converted to the host clock (21:00 on UTC, 04:00 on the current Asia/Jakarta clock); the stack returns through Docker's restart policies. Docker's own repository is not an unattended origin, so Docker is upgraded by hand at a deploy (`sudo apt-get install --only-upgrade docker-ce docker-ce-cli containerd.io docker-compose-plugin`).

Deploy user: `deploy` has no sudo and a locked password. Every key in `~deploy/.ssh/authorized_keys` starts with `restrict,command="/usr/local/bin/rotaria-deploy"`, so it gets no shell and no forwarding; until the deploy command is installed there, that path holds a placeholder that exits 1. `DEPLOY_PUBKEY='ssh-ed25519 ...'` rewrites the file with that one key.

`.env.production` stays mode 0600 (the script enforces it). Its encrypted copy, `age -r <owner's age public key> .env.production`, lives in R2 at `rotaria-backups/env/env.production.<date>.age`, refreshed after every change; neither copy is ever committed.

Cloudflare, zone `rotaria.site` (Free plan, set in the dashboard on 2026-11-07). Tunnel `rotaria-vps2`: its token is `CLOUDFLARE_TUNNEL_TOKEN` in `.env.production`; published application routes `rotaria.site` and `www.rotaria.site` go to `http://edge:80`; there is no `api` hostname (ADR-45). Until `cloudflared` runs on VPS-2 the site answers 530.

| Setting | Value | Why |
|---|---|---|
| SSL/TLS encryption mode | Full (strict) | zone default; Tunnel hostnames do not use it |
| Always Use HTTPS | On | the origin serves plain HTTP on `:80` and never redirects; must stay on for any deploy |
| Minimum TLS version | 1.2 | |
| HTTP/3 | On | |
| DNSSEC | On; DS `2371 13 2 26F407985418052530DCCDC2B913858C6D0D1B8DBDAD752F859FBF5322B3D627` at Namecheap | |
| Bot Fight Mode | Off | cannot be excepted per path and challenges API and app traffic; never put a challenge on `/api/*` |
| Cloudflare managed ruleset | Always active | |
| Leaked credentials mitigation, "Rate limit authentication requests" | not created | either takes the plan's only rate limiting rule |

| Rule | Kind | Expression | Action |
|---|---|---|---|
| `api-versi-tak-dikenal` | custom | `(starts_with(http.request.uri.path, "/api/") and not starts_with(http.request.uri.path, "/api/v1/"))` | Block |
| `metode-tak-dikenal` | custom | `(not http.request.method in {"GET" "HEAD" "POST" "PUT" "PATCH" "DELETE" "OPTIONS"})` | Block |
| `non-api-hanya-baca` | custom | `(not starts_with(http.request.uri.path, "/api/") and not http.request.method in {"GET" "HEAD"})` | Block |
| `auth-burst` | rate limiting | `(starts_with(http.request.uri.path, "/api/v1/auth/"))` | over 30 requests in 10 s per IP: Block for 10 s |
| `api-tanpa-cache` | cache | `(starts_with(http.request.uri.path, "/api/") and not starts_with(http.request.uri.path, "/api/v1/covers/"))` | Bypass cache |
| `wasm-cache` | cache | `(http.request.uri.path.extension eq "wasm")` | Eligible for cache, TTL from the origin |

The Free plan allows 5 custom rules and one rate limiting rule (path only, counted per IP, 10 s period and timeout), so every other limit stays in the server. Cloudflare caches cover images for their full year without a rule; taking one down needs a purge per URL (manual QA, section 3).

Every `uses:` names a full commit SHA with the version in a comment, every checkout sets `persist-credentials: false`, and every `FROM` and third-party production `image:` carries an `@sha256:` digest (the project's own images go by release tag). Trunk is checked against `scripts/trunk.sha256` in CI and in `Dockerfile.edge`; a new `TRUNK_VERSION` needs new lines there (the release's `.sha256` assets).

Repository protection (rulesets, `gh api repos/aprxty3/project-baca/rulesets`): `main` takes changes only through a pull request with the `check`, `backend`, and `web` jobs green, never a force push or a deletion, and the owner (admin role) may bypass; `develop` refuses force pushes and deletion for everyone; tags `v*` can be created only by the owner and never moved or deleted. Dependabot alerts and security updates are on, Dependabot version updates open one grouped minor-and-patch pull request per ecosystem against `develop` each Monday (releases at least 7 days old), CodeQL (`.github/workflows/codeql.yml`: Actions, JavaScript, Python, Rust without a build; on pull requests, pushes to `main`, and Mondays) uploads to the Security tab, and an advisory exception in `deny.toml` carries its reason and review date, and vulnerabilities are reported privately through the Security tab ([SECURITY.md](SECURITY.md)).

Not in CI: visual goldens (font rasterisation differs on the runner), Firefox smoke, and the p95 benchmarks (they follow machine load); run them locally with `make test-web-full` and `make test-performance-release`.

Versions follow Semantic Versioning with the Cargo reading of `0.y.z`: a new `y` means a break, so additions and fixes raise `z`. One release per milestone or batch of work, not per merge.

| Version | When |
|---|---|
| `0.1.z` | Each release before the public beta; API, web and app ship together, so no client breaks |
| `0.2.0` | The public beta opens (M5 done, the launch checklist passes on the production domain) |
| later `0.y.0` | A change readers feel as a new product step (M6), or an API change that breaks installed apps |
| `1.0.0` | The owner declares the product stable and the API contract binding for installed apps |

Cutting a release:

1. Set `[workspace.package] version` in `Cargo.toml` and `version` in `apps/tauri/Cargo.toml` and `apps/tauri/tauri.conf.json` (the Android `versionCode` follows it), rename `## [Unreleased]` in `CHANGELOG.md` to `## [X.Y.Z] — YYYY-MM-DD`, and open a new empty `[Unreleased]`.
2. `python3 scripts/release_notes.py --check-version`, then a pull request from `develop` to `main`, merged once CI is green.
3. `git tag vX.Y.Z && git push origin vX.Y.Z`.

Verifying a released image (anonymous pull works while the packages are public; ADR-40):

```bash
for i in server worker edge; do
  ref=ghcr.io/aprxty3/project-baca/$i:X.Y.Z
  docker buildx imagetools inspect "$ref"   # lists linux/amd64 and linux/arm64
  gh attestation verify "oci://$ref" -R aprxty3/project-baca
  for p in linux/amd64 linux/arm64; do
    docker buildx imagetools inspect "$ref" --format "{{ json (index .SBOM \"$p\") }}" | head -c 120; echo
    docker buildx imagetools inspect "$ref" --format "{{ json (index .Provenance \"$p\") }}" | head -c 120; echo
  done
done
```

Each image carries an SBOM and SLSA provenance in `mode=max`, which records build args, so no secret is ever passed as one.

The edge image is built with `API_BASE_URL=/api/v1`, so one image serves any domain.
