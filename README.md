# Suki

A Davao City market-shopping platform with a responsive Next.js/TypeScript storefront and a Rust/Axum backend backed by MariaDB.

## Start

See the [complete setup guide](docs/SETUP.md) for tool installation, database creation, environment variables, startup, seed data, integrations, tests, backups and deployment.

- Node.js 24+ and npm for the frontend.
- Rust 1.99.1, pinned in `rust-toolchain.toml` (2024 edition).
- MariaDB 11.8 LTS.

Follow [backend setup and API documentation](backend/README.md) to create databases, configure private environment values and start the API. Then:

```sh
npm ci
npm run dev
```

Open http://localhost:3000. Next proxies API requests to the Rust server on port 8080. For a production frontend, use `npm run build && npm start`.

## Working flows

- Backend accounts, Argon2id passwords, secure cookie sessions and role/ownership checks.
- Database-backed vendor catalog, availability controls, search, categories, favorites and basket.
- Server-verified road-distance quotes, exact centavo pricing, minimum-order enforcement, immutable order snapshots and transactional idempotency.
- GCash reference submission and manual administrator verification, refunds, spending approvals and substitutions.
- Courier assignment, actual purchase recording, reconciliation and valid order transitions.
- Persisted order chat with authenticated WebSocket events and reconnect recovery.
- Authorized active-delivery location reporting, timestamps, throttling and termination.
- Vendor item confirmation, saved addresses, admin settings/accounts, expense reports and audit history APIs.
- Versioned migrations, backup/restore scripts and deployment guidance.

The storefront uses backend accounts, catalog, checkout, order history, payment references and customer chat. Cart/favorites remain on the current browser; previous prototype draft orders are not loaded or submitted. Use `scripts/seed-local.py` only for synthetic local catalog fixtures.

## Before taking real orders

Google Maps routing and the operator's GCash QR must be configured; checkout deliberately fails closed without them. Real provider transactions have not been tested. Interactive Google Maps selection/display, courier GPS controls and complete operator dashboards still need frontend work. Backend endpoints are available for those workflows. Production TLS, monitoring, scheduled off-host backups, privacy/retention and operational policies also require deployment configuration. This is not yet a pilot-ready deployment.

## Checks

```sh
npm test
npm run typecheck
npm run build
cd backend
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
# With TEST_DATABASE_URL set to a dedicated database ending in _test:
cargo test --locked -- --include-ignored
```

Frontend photos are stored locally from Unsplash. Google Fonts has a system-font fallback.
