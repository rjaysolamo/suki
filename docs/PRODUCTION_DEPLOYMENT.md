# Suki — Production Deployment Guide

> [!IMPORTANT]
> This guide is specific to deploying **Suki** as described in the codebase at `/Users/rackerjoyjugalbot/suki`. All commands assume you have already cloned the repository onto your production build machine. Read every section before beginning — several steps are **irreversible** if performed out of order.

---

## Table of Contents

1. [Prerequisites](#1-prerequisites)
2. [Server Infrastructure](#2-server-infrastructure)
3. [MariaDB Setup](#3-mariadb-setup)
4. [Environment & Secrets Configuration](#4-environment--secrets-configuration)
5. [Building the API Binary](#5-building-the-api-binary)
6. [Running & Managing the Process](#6-running--managing-the-process)
7. [TLS Termination & Ingress (Nginx)](#7-tls-termination--ingress-nginx)
8. [First-Boot Bootstrap (Admin Account)](#8-first-boot-bootstrap-admin-account)
9. [Google Routes API Setup](#9-google-routes-api-setup)
10. [Vendor Photo Storage](#10-vendor-photo-storage)
11. [Mobile App Builds (Android / iOS)](#11-mobile-app-builds-android--ios)
12. [Scheduled Maintenance Jobs](#12-scheduled-maintenance-jobs)
13. [Backups & Restore Testing](#13-backups--restore-testing)
14. [Database Migration Procedure (Upgrades)](#14-database-migration-procedure-upgrades)
15. [Health Monitoring & Alerting](#15-health-monitoring--alerting)
16. [Pre-Launch Checklist](#16-pre-launch-checklist)

---

## 1. Prerequisites

### On the Build Machine

| Tool | Version | Installation |
|---|---|---|
| Rust toolchain | **1.99.1** (pinned by `rust-toolchain.toml`) | `curl https://sh.rustup.rs -sSf \| sh` |
| Cargo | Included with Rust | — |
| OpenSSL (dev libs) | Any recent | `apt install libssl-dev pkg-config` / `brew install openssl` |

The workspace `rust-toolchain.toml` pins the toolchain automatically on first `cargo` invocation:

```toml
[toolchain]
channel = "1.99.1"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

### On the Production Server

| Requirement | Notes |
|---|---|
| Linux (systemd-based recommended) | Ubuntu 22.04 LTS or Debian 12 tested paths |
| MariaDB **11.8 LTS** | Must match — do not use MySQL or older MariaDB |
| Nginx (or any TLS-terminating reverse proxy) | Required for HTTPS and WebSocket proxying |
| Persistent writable directory for vendor photos | e.g. `/var/lib/suki/vendor-images` |
| A domain name with a valid TLS certificate | Let's Encrypt / Certbot recommended |

---

## 2. Server Infrastructure

### Single-Instance Constraint

> [!WARNING]
> The WebSocket event bus uses an **in-process `tokio::broadcast` channel**. Running multiple `suki-api` instances behind a load balancer will cause WebSocket clients connected to different instances to miss events. Deploy **exactly one** `suki-api` process for the pilot. Horizontal scaling requires adding a shared event broker (e.g. Redis Pub/Sub) first.

### Recommended Topology

```
Internet
   │  HTTPS :443 / WSS :443
   ▼
[ Nginx — TLS termination ]
   │  HTTP :8080 (Unix socket recommended)
   ▼
[ suki-api (single process) ]
   │  MySQL protocol (private network / loopback)
   ▼
[ MariaDB 11.8 LTS ]
```

---

## 3. MariaDB Setup

### 3.1 Install MariaDB 11.8 LTS

```bash
# Ubuntu/Debian — use the official MariaDB repository for 11.8
curl -LsS https://downloads.mariadb.com/MariaDB/mariadb_repo_setup \
  | sudo bash -s -- --mariadb-server-version="mariadb-11.8"
sudo apt update && sudo apt install mariadb-server
sudo systemctl enable --now mariadb
sudo mariadb-secure-installation
```

### 3.2 Create Databases and User

Connect as root:

```bash
sudo mariadb -u root
```

Run the following SQL — replace `CHANGE_ME` with a **strong, randomly generated** password:

```sql
-- Production database
CREATE DATABASE suki
  CHARACTER SET utf8mb4
  COLLATE utf8mb4_unicode_ci;

-- Dedicated application runtime user (loopback only)
CREATE USER 'suki'@'127.0.0.1' IDENTIFIED BY 'CHANGE_ME';
GRANT ALL PRIVILEGES ON suki.* TO 'suki'@'127.0.0.1';

-- Separate test database (for integration tests only — not needed on a production-only server)
-- CREATE DATABASE suki_test CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
-- GRANT ALL PRIVILEGES ON suki_test.* TO 'suki'@'127.0.0.1';

FLUSH PRIVILEGES;
EXIT;
```

> [!NOTE]
> `suki-api` runs SQLx migrations automatically at startup using an internal migration lock. You do **not** need to run migrations manually. The runtime user needs `CREATE TABLE`, `CREATE TRIGGER`, and `ALTER TABLE` during first startup for migrations; afterwards, `SELECT / INSERT / UPDATE / DELETE` is sufficient. For tighter security, use a migration user with elevated privileges for first startup only.

### 3.3 Harden MariaDB

```bash
# Bind to loopback only — edit /etc/mysql/mariadb.conf.d/50-server.cnf
[mysqld]
bind-address = 127.0.0.1

sudo systemctl restart mariadb
```

---

## 4. Environment & Secrets Configuration

### 4.1 Generate the Integrity Key

> [!CAUTION]
> The sample key in `.env` (`dc9dc127...`) **must never be used in production**. Generate a fresh one:

```bash
openssl rand -hex 32
# Example output: a7f3c9d2e1b045...  (store this somewhere safe)
```

- Keep this key in a secrets manager (Vault, AWS Secrets Manager, environment injection) or a `chmod 600` file.
- **Back it up separately from MariaDB.** Losing the key makes all integrity chain verifications fail permanently.
- **All future API instances must use the same key.** Rotating requires a data migration.

### 4.2 Create the Production `.env` File

Place this file in the directory from which you run `suki-api` (e.g. `/etc/suki/`), with mode `600` and owned by the service user:



```bash
sudo mkdir -p /etc/suki
sudo chmod 700 /etc/suki
sudo tee /etc/suki/.env > /dev/null << 'EOF'
# Database
DATABASE_URL=mysql://suki:CHANGE_ME@127.0.0.1:3306/suki

# Required cryptographic signing key — NEVER use the sample value
SUKI_INTEGRITY_KEY=<your-openssl-rand-hex-32-output>
DATA_ENCRYPTION_KEY=<a-different-openssl-rand-hex-32-output>
SESSION_SIGNING_KEY=<another-different-openssl-rand-hex-32-output>

# Server binding
BIND_ADDR=127.0.0.1:8080
APP_ORIGIN=https://your-domain.example

# MUST be true for HTTPS production
COOKIE_SECURE=true

# Google Routes API (required for checkout delivery quotes)
GOOGLE_MAPS_API_KEY=<your-restricted-server-api-key>
GOOGLE_ROUTES_URL=https://routes.googleapis.com/directions/v2:computeRoutes

# Vendor photo storage — must be a writable, persistent path; include in backups
SUKI_MEDIA_DIR=/var/lib/suki/vendor-images

# Logging
RUST_LOG=suki_api=info,tower_http=info
EOF

sudo chmod 600 /etc/suki/.env
sudo chown suki:suki /etc/suki/.env   # assumes a 'suki' service user
```


### 4.3 Create the Service User

```bash
sudo useradd --system --shell /usr/sbin/nologin --home-dir /var/lib/suki --create-home suki
sudo mkdir -p /var/lib/suki/vendor-images
sudo chown -R suki:suki /var/lib/suki
```

### 4.4 All Environment Variables Reference



| Variable | Required | Default | Production value |
|---|---|---|---|
| `DATABASE_URL` | ✅ | — | `mysql://suki:…@127.0.0.1:3306/suki` |
| `SUKI_INTEGRITY_KEY` | ✅ | — | 64-char hex from `openssl rand -hex 32` |
| `DATA_ENCRYPTION_KEY` | ✅ | — | Separate 64-char hex key for MariaDB protected fields |
| `SESSION_SIGNING_KEY` | ✅ | — | Separate 64-char hex key for access JWTs |
| `BIND_ADDR` | No | `127.0.0.1:8080` | `127.0.0.1:8080` (proxied by Nginx) |
| `APP_ORIGIN` | No | `http://localhost:8080` | `https://your-domain.example` (exact, no trailing slash) |
| `COOKIE_SECURE` | No | `true` | `true` |
| `GOOGLE_MAPS_API_KEY` | Required for checkout | `""` (checkout disabled) | Restricted server Routes API key |
| `GOOGLE_ROUTES_URL` | No | `https://routes.googleapis.com/…` | Leave default |
| `SUKI_MEDIA_DIR` | No | `target/suki-media` | `/var/lib/suki/vendor-images` |
| `RUST_LOG` | No | `suki_api=info` | `suki_api=info,tower_http=info` |
| `SUKI_ALLOW_LEGACY_QUOTES` | No | `false` (disabled) | Omit (keep disabled in production) |
| `BOOTSTRAP_ADMIN_EMAIL` | First run only | — | Set once; **remove after first successful start** |
| `BOOTSTRAP_ADMIN_PASSWORD` | First run only | — | Min 16 chars; **remove after first successful start** |


---

## 5. Building the API Binary

Build on the same OS / architecture as the production server (or use cross-compilation):

```bash
# From the workspace root
cargo build --release --locked -p suki-backend
```

The binary is at `target/release/suki-api`. Copy it to the server:

```bash
scp target/release/suki-api user@your-server:/usr/local/bin/suki-api
ssh user@your-server "sudo chmod 755 /usr/local/bin/suki-api"
```

> [!TIP]
> For reproducible builds, use the same `rust-toolchain.toml`-pinned version. Run `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo test --locked` before promoting any binary to production.

---

## 6. Running & Managing the Process

### 6.1 systemd Service Unit

```bash
sudo tee /etc/systemd/system/suki-api.service > /dev/null << 'EOF'
[Unit]
Description=Suki API (suki-api)
After=network.target mariadb.service
Requires=mariadb.service

[Service]
Type=simple
User=suki
Group=suki
WorkingDirectory=/etc/suki
EnvironmentFile=/etc/suki/.env
ExecStart=/usr/local/bin/suki-api
Restart=on-failure
RestartSec=5s
# Graceful shutdown — suki-api listens for SIGINT (Ctrl-C) via tokio::signal::ctrl_c
KillSignal=SIGINT
TimeoutStopSec=30s

# Hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=/var/lib/suki

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable suki-api
sudo systemctl start suki-api
sudo systemctl status suki-api
```

### 6.2 Verify Startup

```bash
# Should return {"status":"ok"}
curl --fail http://127.0.0.1:8080/api/health
```

The binary applies all pending migrations before it begins accepting connections, so the first startup may take a few seconds.

### 6.3 View Logs

```bash
sudo journalctl -u suki-api -f
```

---

## 7. TLS Termination & Ingress (Nginx)

### 7.1 Install Certbot & Obtain Certificate

```bash
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d your-domain.example
```

### 7.2 Nginx Configuration

```nginx
# /etc/nginx/sites-available/suki

# Redirect HTTP → HTTPS
server {
    listen 80;
    server_name your-domain.example;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your-domain.example;

    ssl_certificate     /etc/letsencrypt/live/your-domain.example/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.example/privkey.pem;
    ssl_protocols       TLSv1.2 TLSv1.3;
    ssl_ciphers         HIGH:!aNULL:!MD5;

    # Rate limiting — protect registration and login endpoints
    limit_req_zone $binary_remote_addr zone=auth:10m rate=5r/m;
    limit_req_zone $binary_remote_addr zone=api:10m rate=60r/m;

    # Body size limit consistent with the API's 32 KiB cap
    client_max_body_size 6m;  # 5 MiB photo + overhead

    # API (REST)
    location /api/ {
        limit_req zone=api burst=20 nodelay;
        proxy_pass         http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header   Host $host;
        proxy_set_header   X-Real-IP $remote_addr;
        proxy_set_header   X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header   X-Forwarded-Proto $scheme;
        proxy_read_timeout 30s;
    }

    # Authentication endpoints — stricter rate limit
    location ~ ^/api/auth/(register|login) {
        limit_req zone=auth burst=5 nodelay;
        proxy_pass         http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header   Host $host;
    }

    # WebSocket endpoints — /api/orders/:id/events and /api/orders/:id/chat/ws
    location ~ ^/api/orders/[^/]+/(events|chat/ws)$ {
        proxy_pass             http://127.0.0.1:8080;
        proxy_http_version     1.1;
        proxy_set_header       Upgrade    $http_upgrade;
        proxy_set_header       Connection "upgrade";
        proxy_set_header       Host       $host;
        proxy_set_header       Origin     $http_origin;
        proxy_read_timeout     3600s;  # keep WS alive
        proxy_send_timeout     3600s;
    }

    # Vendor images
    location /images/ {
        proxy_pass         http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_cache_valid  200 1d;
        add_header         Cache-Control "public, max-age=86400";
    }
}
```

```bash
sudo ln -s /etc/nginx/sites-available/suki /etc/nginx/sites-enabled/suki
sudo nginx -t
sudo systemctl reload nginx
```

> [!IMPORTANT]
> The API validates both the `X-Suki-Request` header and the `Origin` header against `APP_ORIGIN` for every mutating request. Nginx must pass the `Origin` header through unchanged — do **not** strip it.

---

## 8. First-Boot Bootstrap (Admin Account)

Before the first real startup, temporarily add bootstrap credentials to `/etc/suki/.env`:

```bash
# Add to /etc/suki/.env — TEMPORARY; remove immediately after first successful start
BOOTSTRAP_ADMIN_EMAIL=operator@your-domain.example
BOOTSTRAP_ADMIN_PASSWORD=YourStrongAdminPassword16+chars
```

```bash
sudo systemctl restart suki-api
# Wait for startup
curl --fail http://127.0.0.1:8080/api/health

# IMMEDIATELY remove bootstrap lines from .env
sudo sed -i '/BOOTSTRAP_ADMIN/d' /etc/suki/.env
sudo systemctl restart suki-api
```

> [!CAUTION]
> Bootstrap runs only when **no admin account exists**. If you forget to remove these lines they are harmless after the first admin is created, but keeping credentials in a file is a security risk. Remove them immediately.

Log in as the admin at `https://your-domain.example` to:
- Activate vendor accounts that have registered and are awaiting approval.
- Configure the GCash QR URL (via `PUT /api/settings`).
- Set the service area bounding box (`south`, `north`, `west`, `east`, `max_distance_meters`).
- Review the pickup origin coordinates (`pickup_lat`, `pickup_lng`).

---

## 9. Google Routes API Setup

### 9.1 Create a Restricted Server Key

1. Go to [Google Cloud Console → APIs & Services → Credentials](https://console.cloud.google.com/apis/credentials).
2. Create an **API key**, restrict it to:
   - **Application restrictions**: IP addresses → add your server's public IP.
   - **API restrictions**: Restrict to **Routes API** only.
3. Enable the **Routes API** in your Google Cloud project.
4. Set in `/etc/suki/.env`:
   ```
   GOOGLE_MAPS_API_KEY=AIza...your-key...
   ```

### 9.2 Review the Service Area

The default routing origin is **Bankerohan Market, Davao City** (`7.0644°N, 125.6027°E`). The default bounding box covers Davao (`south=6.9, north=7.3, west=125.3, east=125.75`) with a 30 km maximum distance.

Adjust these defaults via the admin settings endpoint before accepting real orders:

```bash
# Get current settings
curl -s https://your-domain.example/api/settings | python3 -m json.tool

# Update via admin session (use the actual admin session cookie)
curl -X PUT https://your-domain.example/api/settings \
  -H "Content-Type: application/json" \
  -H "X-Suki-Request: 1" \
  -H "Cookie: suki_session=<your-admin-session>" \
  -d '{
    "base_fee_centavos": 6000,
    "per_km_centavos": 350,
    "minimum_centavos": 45000,
    "pickup_lat": 7.0644,
    "pickup_lng": 125.6027,
    "south": 6.9,
    "north": 7.3,
    "west": 125.3,
    "east": 125.75,
    "max_distance_meters": 30000,
    "gcash_qr_url": "/images/gcash-qr.png",
    "chat_retention_days": 30
  }'
```

> [!NOTE]
> `gcash_qr_url` must be a local `/images/` path pointing to an image you have uploaded to `SUKI_MEDIA_DIR`. The weighted delivery quote formula uses a **fixed** rate (₱85 minimum, ₱7/km, ₱5/kg) from Bankerohan — these cannot be changed via settings; they are compiled constants in `suki-shared`.

---

## 10. Vendor Photo Storage

```bash
# The directory must exist and be owned by the suki service user
sudo mkdir -p /var/lib/suki/vendor-images
sudo chown -R suki:suki /var/lib/suki/vendor-images
sudo chmod 750 /var/lib/suki/vendor-images
```

The API re-encodes uploaded JPEG/PNG files via the `image` crate before storing. Replaced files are retained (historical URLs remain readable); unreferenced files must be cleaned up offline by an operator.

> [!IMPORTANT]
> **Back up `/var/lib/suki/vendor-images/` alongside MariaDB.** A database restore without the matching media directory leaves broken image URLs for all products.

---

## 11. Mobile App Builds (Android / iOS)

Build on a macOS machine for iOS or any machine with the Android SDK for Android.

### 11.1 Install Build Tools

```bash
cargo install dioxus-cli --version 0.7.10 --locked
rustup target add aarch64-linux-android   # Android 64-bit ARM
rustup target add aarch64-apple-ios       # iOS 64-bit (device)
rustup target add aarch64-apple-ios-sim   # iOS 64-bit (simulator)
```

**Android additional requirements:**
- JDK 17
- Android SDK platform 35, build-tools 35.0.0, NDK **27.2.12479018**
- `ANDROID_HOME` and `NDK_HOME` set in your shell

### 11.2 Build Android APK

```bash
export SUKI_API_URL=https://your-domain.example
# Optional: override WebSocket Origin if APP_ORIGIN differs
# export SUKI_APP_ORIGIN=https://your-domain.example

cd crates/suki-mobile
dx build --platform android \
         --target aarch64-linux-android \
         --features mobile \
         --release \
         --locked
```

> [!NOTE]
> The `build.rs` emits `-Wl,-z,max-page-size=16384` and `-Wl,-z,common-page-size=16384` for NDK r27 compatibility with Android devices that use 16 KB memory pages (Android 15+ requirement). Do not remove `build.rs`.

`SUKI_API_URL` is **baked at compile time** via `option_env!`. Every production binary must be built with the correct production URL.

### 11.3 Build iOS App

```bash
export SUKI_API_URL=https://your-domain.example
cd crates/suki-mobile

# Device build (requires Xcode signing)
dx build --platform ios \
         --target aarch64-apple-ios \
         --features mobile \
         --release \
         --locked

# Simulator build
dx build --platform ios \
         --target aarch64-apple-ios-sim \
         --features mobile \
         --locked
```

iOS uses **foreground Core Location** (`CLLocationManager`). Location sharing stops when the app leaves the foreground.

### 11.4 App Configuration (`Dioxus.toml`)

```toml
[application]
name = "Suki Davao"
default_platform = "android"
android_min_sdk_version = 28       # Android 9+

[android]
min_sdk = 28
target_sdk = 35
compile_sdk = 35

[android.application]
uses_cleartext_traffic = false     # enforces HTTPS

[permissions]
location = { precision = "fine", description = "Share your location with the customer during an active delivery." }
```

---

## 12. Scheduled Maintenance Jobs

### 12.1 Daily Prune Job

```bash
# /etc/suki/prune.sql is a copy of crates/suki-backend/scripts/prune.sql
sudo cp /path/to/suki/crates/suki-backend/scripts/prune.sql /etc/suki/prune.sql
```

Create a daily cron entry for the `suki` user (or a dedicated maintenance DB user):

```bash
sudo tee /etc/cron.d/suki-prune > /dev/null << 'EOF'
# Daily prune at 02:00 — run AFTER nightly backup
0 2 * * * suki mariadb --defaults-file=/etc/suki/.mariadb-maintenance.cnf suki < /etc/suki/prune.sql
EOF
```

The prune script removes:
- Expired sessions
- Old login attempt counters (>24 h after reset)
- Expired delivery quotes (>24 h after expiry)
- Chat messages beyond `chat_retention_days` for completed/canceled orders

> [!CAUTION]
> The prune script **does not** delete financial records, orders, audit logs, or user accounts. Agree on legal retention periods and account erasure procedures (Philippine DPA / GDPR) with your legal team before going live. Do not delete financial evidence ad hoc.

### 12.2 Backup Credentials File

```bash
sudo tee /etc/suki/.mariadb-backup.cnf > /dev/null << 'EOF'
[client]
user=suki-backup
password=BACKUP_USER_PASSWORD
host=127.0.0.1
EOF
sudo chmod 600 /etc/suki/.mariadb-backup.cnf
sudo chown suki:suki /etc/suki/.mariadb-backup.cnf
```

---

## 13. Backups & Restore Testing

### 13.1 Nightly Backup

```bash
# /usr/local/bin/suki-backup.sh
#!/bin/bash
set -euo pipefail
BACKUP_DIR=/var/backups/suki
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
mkdir -p "$BACKUP_DIR"

# Database dump
mariadb-dump \
  --defaults-file=/etc/suki/.mariadb-backup.cnf \
  --single-transaction \
  --routines \
  --triggers \
  suki | gzip > "$BACKUP_DIR/suki_${TIMESTAMP}.sql.gz"

# Media directory (vendor photos)
tar -czf "$BACKUP_DIR/suki-media_${TIMESTAMP}.tar.gz" \
  -C /var/lib/suki vendor-images

# Retain last 14 days
find "$BACKUP_DIR" -name "*.gz" -mtime +14 -delete

echo "Backup complete: $BACKUP_DIR/suki_${TIMESTAMP}.sql.gz"
```

```bash
sudo chmod 755 /usr/local/bin/suki-backup.sh

# Schedule nightly at 01:00
sudo tee /etc/cron.d/suki-backup > /dev/null << 'EOF'
0 1 * * * suki /usr/local/bin/suki-backup.sh >> /var/log/suki-backup.log 2>&1
EOF
```

**Off-host copy:** Encrypt and ship backups off-site (S3, Backblaze B2, etc.). A backup that lives on the same server as the data is not a backup.

### 13.2 Regular Restore Testing

Monthly, restore into a temporary database ending in `_restore` and verify:

```bash
# Create a restore target (empty)
mariadb -u root -e "CREATE DATABASE suki_restore CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"

# Restore
zcat /var/backups/suki/suki_LATEST.sql.gz | mariadb -u root suki_restore

# Compare table counts
mariadb -u root -e "
  SELECT table_name, table_rows
  FROM information_schema.tables
  WHERE table_schema IN ('suki', 'suki_restore')
  ORDER BY table_name, table_schema;
"

# Compare key financial totals
mariadb -u root suki_restore -e "
  SELECT
    (SELECT COUNT(*) FROM orders) AS orders,
    (SELECT CAST(COALESCE(SUM(received_amount_centavos),0) AS SIGNED) FROM payments WHERE status='paid') AS total_received_centavos,
    (SELECT COUNT(*) FROM users) AS users;
"

# Drop restore database when done
mariadb -u root -e "DROP DATABASE suki_restore;"
```

---

## 14. Database Migration Procedure (Upgrades)

> [!WARNING]
> Migrations 0004 and 0005 install database triggers. Partial application of triggers can leave the database in an inconsistent state. Follow this procedure precisely.

### Safe Upgrade Steps

```
1. Take a validated database backup (run restore test on a copy).
2. Stop ALL suki-api instances.
3. Keep all writers stopped until the upgraded API passes its health check.
4. Deploy the new binary to /usr/local/bin/suki-api.
5. sudo systemctl start suki-api
6. Watch logs: sudo journalctl -u suki-api -f
7. curl --fail http://127.0.0.1:8080/api/health
8. Resume traffic only after health check passes.
```

### If Migration Fails at Startup

Do **not** retry, truncate tables, or manually mark migrations complete. Instead:

```sql
-- Check which migrations have applied
SELECT version, success, description FROM _sqlx_migrations ORDER BY version;

-- Check which schema objects exist
SHOW TABLES LIKE 'order_integrity';
SHOW TABLES LIKE 'order_snapshots';
SHOW COLUMNS FROM payments LIKE 'reference_hash';
SHOW COLUMNS FROM chat_messages LIKE 'signature';
SHOW TRIGGERS;
```

Then restore your pre-upgrade backup to a fresh database, verify it, and run the migration against that database before switching the API.

---

## 15. Health Monitoring & Alerting

### 15.1 Health Endpoint

```bash
# Returns {"status":"ok"} when the API and DB connection are healthy
curl --fail https://your-domain.example/api/health
```

Set up an external uptime monitor (UptimeRobot, Pingdom, Grafana Cloud) to poll `/api/health` every minute and alert on failure.

### 15.2 systemd Process Monitoring

```bash
# Alert if the service is not active
systemctl is-active suki-api || send_alert "suki-api is down"
```

### 15.3 Key Log Signals

| Log pattern | Meaning |
|---|---|
| `Suki API ready address=…` | Successful startup |
| `database operation failed error=…` | DB connectivity issue |
| `error=…` in route handler | Application-level error (review for data issues) |
| `tower_http` 5xx entries | Unhandled errors reaching the router |

### 15.4 Admin Operational Checks

After deployment and regularly during operations:
- Review failed payment verifications via `GET /api/admin/payments/pending`.
- Check the audit log for unexpected actions via `GET /api/admin/audit`.
- Run the order integrity inspector on suspicious orders via `GET /api/admin/orders/:id/integrity`.
- Monitor `GET /api/admin/reports` for negative margins (refunds exceeding delivery fees).

---

## 16. Pre-Launch Checklist

Work through every item before accepting real orders or customer data.

### Infrastructure
- [ ] MariaDB 11.8 LTS running and bound to loopback only
- [ ] `suki-api` binary built with `--release --locked` from the correct Rust 1.99.1 toolchain
- [ ] `SUKI_INTEGRITY_KEY` is a **fresh** 64-char hex value — not the sample from `.env`
- [ ] `/etc/suki/.env` has mode `600` and is owned by the service user
- [ ] `COOKIE_SECURE=true` confirmed in `.env`
- [ ] `APP_ORIGIN` matches the exact HTTPS origin (no trailing slash)
- [ ] Bootstrap admin credentials **removed** from `.env` after first start

### TLS & Ingress
- [ ] Valid TLS certificate installed and auto-renewing (Certbot timer enabled)
- [ ] Nginx passes `Origin` header through unchanged
- [ ] WebSocket proxy (`Upgrade`/`Connection` headers) tested end-to-end
- [ ] Rate limiting configured for `/api/auth/register` and `/api/auth/login`
- [ ] HTTP redirects to HTTPS (no mixed-content)

### External Services
- [ ] Google Routes API key is **IP-restricted** and **API-restricted** to Routes API only
- [ ] Delivery quote tested end-to-end (verify `distance_meters` is non-zero)
- [ ] Service area bounding box reviewed against actual coverage
- [ ] GCash QR image uploaded and `gcash_qr_url` set in settings

### Storage & Backups
- [ ] `SUKI_MEDIA_DIR` is a persistent writable path (`/var/lib/suki/vendor-images`)
- [ ] Nightly backup script tested and scheduled (DB + media)
- [ ] Off-host backup copy configured and tested
- [ ] Restore test completed on a copy of the backup
- [ ] `SUKI_INTEGRITY_KEY` backed up **separately** from MariaDB

### Mobile Apps
- [ ] Android APK built with `SUKI_API_URL=https://your-domain.example` (no trailing slash)
- [ ] `uses_cleartext_traffic = false` confirmed in `Dioxus.toml`
- [ ] Location permission tested on a real device
- [ ] WebSocket connection tested on-device (not simulator)
- [ ] iOS app tested in foreground (location stops in background by design)

### Operations
- [ ] Admin account created and bootstrap credentials removed
- [ ] At least one vendor account activated
- [ ] Prune cron scheduled (after backup, not before)
- [ ] `GET /api/health` monitoring configured with alerting
- [ ] Operational procedures documented for: payment disputes, refunds, substitutions, cancellations, vendor settlement
- [ ] Real Google Routes and GCash transactions verified end-to-end before accepting customer payments
- [ ] Legal review complete: Philippine DPA compliance, data retention, refund policy

---

> [!NOTE]
> GCash payments in Suki are **manually verified by an administrator or assigned rider**. The API records references and amounts, but it does not connect to the GCash API. Ensure your operations team has clear procedures for verifying GCash transfer references and handling disputes before launch.
