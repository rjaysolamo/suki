# Suki setup guide

Run commands from the repository root unless a section says otherwise. The application needs three processes: **MariaDB → Rust API → Next.js**. No Docker, Redis, or separate message broker is required for this MVP.

## 1. Install the tools

| Tool | Version | Purpose |
| --- | --- | --- |
| Rust / Cargo | 1.99.1, selected by `rust-toolchain.toml` | Backend, migrations, tests |
| MariaDB | 11.8 LTS; 11.8.8 verified | Persistent application data |
| Node.js | 24 or newer | Frontend and pricing tests |
| npm | Included with Node; 11.11.0 verified | Frontend dependencies |
| Python | 3.9 or newer, optional | Synthetic local catalog seed |

On a normal Amazon Linux 2023 host:

```bash
sudo dnf install -y gcc gcc-c++ make pkgconf-pkg-config mariadb118-server mariadb118
sudo systemctl enable --now mariadb
```

Install Node 24 using your existing Node version manager. For example, if mise is installed:

```bash
mise use --global node@24
```

Install Rust through [rustup](https://rustup.rs/). Then, inside this repository:

```bash
rustup show
cargo --version
node --version
npm --version
mariadb --version
```

The repository automatically selects Rust 1.99.1 and the Rust 2024 edition. Crate versions are locked in `backend/Cargo.lock`.

The current cloud sandbox already has these tools. It uses a private MariaDB datadir rather than systemd; use the sandbox commands in [section 10](#10-resume-the-current-cloud-sandbox).

## 2. Create the databases

Open an administrator SQL session (`sudo mariadb` on a standard installation) and run:

```sql
CREATE DATABASE suki CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE DATABASE suki_test CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE USER 'suki'@'127.0.0.1' IDENTIFIED BY 'REPLACE_WITH_PRIVATE_DATABASE_PASSWORD';
GRANT ALL PRIVILEGES ON suki.* TO 'suki'@'127.0.0.1';
GRANT ALL PRIVILEGES ON suki_test.* TO 'suki'@'127.0.0.1';
```

Choose a private password and replace the placeholder before running the SQL. A generated hexadecimal password avoids URL escaping when used in `DATABASE_URL`. Test data belongs only in `suki_test`; the integration test refuses database names that do not end in `_test`.

These local grants allow startup migrations. For production, separate migration and runtime database privileges as described in [backend operations](../backend/README.md#deployment-backups-and-operations).

## 3. Configure the API

```bash
cp backend/.env.example backend/.env
chmod 600 backend/.env
```

Edit `backend/.env` privately. Replace `CHANGE_ME` with the database password in both URLs. URL-encode reserved characters if your password is not URL-safe. When shell-loading this file, quote values that contain shell-special characters.

| Variable | Required / default | Meaning |
| --- | --- | --- |
| `DATABASE_URL` | Required | Application MariaDB URL, ending in `/suki` |
| `TEST_DATABASE_URL` | Integration tests only | Dedicated database URL, ending in `/suki_test` |
| `BIND_ADDR` | `127.0.0.1:8080` | API listener |
| `APP_ORIGIN` | `http://localhost:3000` | Exact browser origin, including scheme and port |
| `COOKIE_SECURE` | Defaults to `true` | Set `false` only for local HTTP on `localhost`; use `true` with production HTTPS |
| `RUST_LOG` | `suki_api=info` | Optional tracing filter |
| `BOOTSTRAP_ADMIN_EMAIL` | First startup only | Creates the first administrator when none exists |
| `BOOTSTRAP_ADMIN_PASSWORD` | With bootstrap email | Private password, 16–128 bytes; remove after first startup |
| `GOOGLE_MAPS_API_KEY` | Required for delivery quotes | Server-only Google Routes key |
| `GOOGLE_ROUTES_URL` | Google Routes computeRoutes endpoint | Provider endpoint override; normally leave unset |

Set the two bootstrap administrator values for your first startup. The administrator is not a publicly registered role. Existing accounts are not reset when bootstrap variables are supplied again.

## 4. Start the API and frontend

Terminal 1, from the repository root:

```bash
cd backend
cargo check --locked
cargo run --locked
```

Running inside `backend/` loads `backend/.env` automatically. Startup applies `backend/migrations/` before opening the listener. There is no separate SQLx CLI installation or migration command to run.

Terminal 2, from the repository root:

```bash
npm ci
npm run dev
```

Open **http://localhost:3000**. Verify both API access paths:

```bash
curl --fail http://127.0.0.1:8080/api/health
curl --fail http://localhost:3000/api/health
```

Both should return `{"status":"ok"}`. Health checks include the database connection.

Next forwards `/api/*`, including WebSocket upgrades, to the Rust server. If the API is hosted elsewhere, set `API_INTERNAL_URL` **on the Next process** before starting/building it; default: `http://127.0.0.1:8080`. This is not a `NEXT_PUBLIC_*` variable. Keep provider keys, passwords and session tokens out of browser configuration.

Use Ctrl-C to stop the API or frontend in its terminal. Normal starts reuse dependencies and database contents; do not rerun database creation or initialization on every restart.

## 5. Accounts and optional sample products

- Customers register and sign in through the storefront.
- Vendors register with a shop name. An administrator must activate them before login.
- Administrators use `PATCH /api/admin/users/{id}` to activate vendor accounts or promote an existing account to courier. Account changes revoke existing sessions, so the affected user must sign in again.
- An administrator may also be assigned as the courier.

The full [API reference](../backend/README.md#api) describes operator requests. Mutations require `X-Suki-Request: 1` and an authenticated session cookie where applicable. Browser requests must use the configured origin.

For synthetic local catalog data only, load your **own, trusted, shell-compatible** environment file and run:

```bash
set -a
source backend/.env
set +a
python3 scripts/seed-local.py
unset BOOTSTRAP_ADMIN_PASSWORD
```

The seed script requires the bootstrap administrator email/password, an already-running API, and a loopback `API_URL` (default `http://127.0.0.1:8080`). It creates vendors and products through the API. Generated vendor credentials are saved privately to `$HOME/runtime/suki-api/seed-accounts.json`; override with `SEED_CREDENTIALS_FILE` if needed. It does not configure GCash or Maps. Do not replace that credentials file when reseeding an existing database.

Remove bootstrap settings from the API environment after setup. Keep a private operator record of your administrator credentials; there is no password-reset workflow yet.

## 6. Enable delivery quotes and payments

### Google Maps

1. Enable the Google Routes API in your Google Cloud project and configure billing.
2. Create a restricted server API key and set `GOOGLE_MAPS_API_KEY` in `backend/.env`.
3. Restart the API.
4. Review `GET /api/settings` and update the Bankerohan pickup coordinates, coverage bounds, maximum distance and fees as an administrator.
5. In checkout, enter a delivery address and confirm coordinates, or grant device location permission. Request the road-distance quote before placing an order.

Defaults: **₱450 goods minimum**, **₱60 base delivery fee**, **₱3.50 per road kilometer**. The server computes the fee and revalidates checkout; the browser cannot supply a trusted distance or fee.

The current frontend supports coordinate entry and device location. An interactive Google address picker and tracking map still require frontend work. The local Google emulator does not support Routes; no emulator should be used as a substitute for real delivery quotes.

### GCash

1. Place the operator's actual QR image under `public/images/`.
2. Sign in as administrator and update `gcash_qr_url` using `PUT /api/settings`, for example `/images/gcash-qr.png`. Send the complete settings object returned by GET, with the desired changes.
3. Customers place an order and submit its outstanding amount and transaction reference.
4. The administrator checks the actual GCash transaction, then calls `POST /api/payments/{id}/verify` with the exact received amount.

Submitting a reference does not mark an order paid. Both valid routing and a configured QR are required before order creation. Payments and refunds remain manual external operations; Suki records their verification and audit history. Do not upload a sample payment QR for real customers.

## 7. Run checks

From the repository root:

```bash
cargo check --locked --manifest-path backend/Cargo.toml
cargo fmt --manifest-path backend/Cargo.toml --check
cargo clippy --locked --manifest-path backend/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path backend/Cargo.toml
npm test
npm run typecheck
npm run build
```

The default Rust test command runs unit tests and reports the database integration test as ignored. To include it:

```bash
set -a
source backend/.env
set +a
cargo test --locked --manifest-path backend/Cargo.toml -- --include-ignored
```

This requires `TEST_DATABASE_URL` and a running MariaDB instance. The test applies migrations, creates synthetic accounts/orders, and starts its own synthetic routing fixture. The application API and real Google credentials are not required for that test. Test fixtures remain in the dedicated test database; never point it at application data.

## 8. Backups, restore and retention

Create a private MariaDB client file outside the checkout:

```ini
[client]
user=your_backup_user
password=your_private_backup_password
host=127.0.0.1
port=3306
```

Restrict it with `chmod 600`. Use a backup user with sufficient permissions for the database, routines and triggers. Then:

```bash
export MYSQL_CNF=/private/path/mariadb-backup.cnf
export DATABASE_NAME=suki
bash backend/scripts/backup.sh /private/backups/suki.sql.gz
```

The output directory must already exist. The script writes a compressed transaction-consistent dump atomically and checks gzip integrity.

For a restore exercise, have the database administrator create a **separate empty** database ending in `_restore`, then:

```bash
export RESTORE_DATABASE=suki_restore
bash backend/scripts/restore-test.sh /private/backups/suki.sql.gz
```

The restore script refuses a nonempty target or an unsafe database name. Compare source/restored table counts and payment totals before considering the backup usable. Schedule nightly backups, encrypted off-host storage and regular restore exercises before launch.

Run `backend/scripts/prune.sql` daily with an appropriately restricted maintenance account. It expires sessions and old quote/attempt records and removes chat beyond the configured lifecycle retention. Live locations are deleted on delivery/cancellation. The operator still needs an approved retention/erasure policy for accounts and financial records.

## 9. Production startup

Build once:

```bash
cargo build --release --locked --manifest-path backend/Cargo.toml
npm ci
npm run build
```

Start MariaDB, then `backend/target/release/suki-api` with a protected environment, then `npm start`. A binary started from the repository root does not automatically read `backend/.env`: inject its environment through your process manager, or start it from `backend/` using `./target/release/suki-api`.

Production needs an HTTPS origin, `COOKIE_SECURE=true`, private database/API connectivity, restricted Google keys, the actual GCash QR, TLS termination and WSS upgrade forwarding, ingress rate limits, durable database storage, health monitoring and scheduled backups. Run one backend instance initially; its live broadcasts are in process, while messages/orders persist in MariaDB. More instances require shared event delivery.

Migrations run at API startup; back up before upgrades. Stop the API gracefully with SIGINT. Complete real Google/GCash acceptance tests and operating policies before taking orders. Full courier/admin dashboards and courier device GPS controls remain frontend work.

## 10. Resume the current cloud sandbox

Existing private paths:

- MariaDB datadir: `/home/vercel-sandbox/runtime/suki-db`
- API environment: `/home/vercel-sandbox/runtime/suki-api/backend.env`
- Generated seed accounts: `/home/vercel-sandbox/runtime/suki-api/seed-accounts.json`

Reuse the existing database. Start only stopped processes:

```bash
mariadbd --datadir=/home/vercel-sandbox/runtime/suki-db \
  --socket=/home/vercel-sandbox/runtime/suki-db/mysql.sock \
  --pid-file=/home/vercel-sandbox/runtime/suki-db/mysql.pid \
  --log-error=/home/vercel-sandbox/runtime/suki-db/mysql.log \
  --bind-address=127.0.0.1 --port=3306
```

In a second terminal, from the repository root:

```bash
set -a
source /home/vercel-sandbox/runtime/suki-api/backend.env
set +a
cargo run --locked --manifest-path backend/Cargo.toml
```

In a third terminal:

```bash
npm run dev
```

Wait for `/api/health` to succeed before using the app. The sandbox's real Maps key and GCash QR are intentionally unconfigured, so a local storefront can load while checkout remains unavailable.

## Troubleshooting

| Symptom | Check / fix |
| --- | --- |
| `cargo: command not found` | Load `$HOME/.cargo/env`, or reopen your shell after rustup installation |
| Missing `DATABASE_URL` | Run from `backend/` to load its `.env`, or explicitly load the private environment |
| Database connection refused | Start MariaDB and verify port, bind address and URL |
| Database access denied | Check the password and the database user's `127.0.0.1` grants |
| Migration failure | Read the error and inspect `_sqlx_migrations`; keep applied migration files unchanged and fix through a new migration |
| Frontend API returns 502 | Start the Rust API; confirm `API_INTERNAL_URL` and direct `/api/health` |
| Login succeeds but session does not persist locally | Use `http://localhost:3000`, matching `APP_ORIGIN`, and local-only `COOKIE_SECURE=false` |
| Mutation or WebSocket returns 403 | Check account role/ownership, exact Origin, session cookie, and mutation custom header |
| Vendor cannot log in | Have an administrator activate the account, then sign in again |
| Checkout reports missing routing/QR | Configure Google Routes and operator GCash settings; do not enter an unverified delivery fee |
| Price changed at checkout | Refresh the catalog and review the basket before retrying |
| Port already in use | Reuse the healthy process or stop the task-owned process before restarting |
| `npm start` reports no production build | Run `npm run build`, or use `npm run dev` for development |
