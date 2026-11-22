# Suki setup

## API and MariaDB

Install Rust through [rustup](https://rustup.rs/). Use MariaDB 11.8 LTS. Create UTF-8 databases and a private local user:

```sql
CREATE DATABASE suki CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE DATABASE suki_test CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE USER 'suki'@'127.0.0.1' IDENTIFIED BY 'REPLACE_WITH_PRIVATE_PASSWORD';
GRANT ALL PRIVILEGES ON suki.* TO 'suki'@'127.0.0.1';
GRANT ALL PRIVILEGES ON suki_test.* TO 'suki'@'127.0.0.1';
```

```sh
cd crates/suki-backend
cp .env.example .env
chmod 600 .env
# Edit .env with your private database settings.
cargo run --locked
```


The API applies migrations before listening. Check `curl --fail http://localhost:8080/api/health`. From the workspace root use `cargo run --locked -p suki-backend` with variables supplied by the shell or your process manager. The sample environment describes administrator bootstrap. Remove bootstrap credentials after first startup.




Set `SUKI_INTEGRITY_KEY` before starting or upgrading the API. Generate it once with `openssl rand -hex 32`, keep it in your server secret store and back it up separately from MariaDB. All API instances must use the same key. Replacing it without a migration makes existing signatures fail verification. The migration requires permission to create triggers. Apply schema upgrades with older API instances stopped.

## Account approvals and protected fields

Set `DATA_ENCRYPTION_KEY` to an independently generated 32-byte key (`openssl rand -hex 32`) before starting the API. Keep it in the server secret store and back it up separately from MariaDB. Every instance sharing a database needs the same key. Startup rejects a missing, malformed, or mismatched key. Replacing the key requires a planned re-encryption migration.

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:60845c05e6ac4f28ef05605c2151c3516afe3b78324fcb41f0aeeb0f64077bc7 -->

Customers can sign in after registering. Vendor and rider applications start pending. An administrator whose email is on the allowlist can approve or reject applications in **Admin dashboard → Pending Approvals**. The queue lists applicants oldest first and shows their registration time in Philippine time. The Operations tab contains existing account and role management tools. For a separately hosted web frontend, proxy both `/admin` and `/admin/*` to Axum on the same origin so direct navigation uses the same access guard as the API. Approval unlocks sign-in and creates a durable decision shown as an in-app notice at sign-in. Rejection keeps sign-in blocked. The admin API allows ten requests per authenticated administrator per minute on each API instance.

<!-- /coderabbit-ai-generated:v1 -->

Stop all old API instances and other writers, verify a backup, and allow downtime for migration 0008 and its transactional encryption backfill before serving traffic. The migration keeps the original receipt signatures and audit tags. MariaDB DDL can commit independently; if the schema migration stops partway through, follow the recovery procedure below. If the subsequent application backfill fails, it rolls back and startup stays offline. Use the same encryption key on retry. Old database backups, binary logs, and storage snapshots can still contain plaintext; retain or expire them under your existing access policy.

Encryption covers saved address labels and locations, quote and order delivery locations, checkout totals, and signed receipt payloads. Access to existing order and receipt endpoints keeps the same authenticated order permissions. Other accounting fields, including item prices, payment records and adjustments, remain queryable and may let a reader reconstruct totals. This change does not encrypt the whole database.

**Recover an interrupted integrity migration.** MariaDB commits schema changes separately, so startup can leave part of migration 0004, 0005, or 0008 applied. Before upgrading, stop all API instances and other writers and take a validated database backup. Keep writes stopped until the migrated API passes its health check.

If startup fails during migration, stop retries and keep the failed database for inspection. Check the migration records and which schema objects already exist:

```sql
SELECT version, success FROM _sqlx_migrations WHERE version IN (4, 5, 8);
SHOW TABLES LIKE 'order_integrity';
SHOW TABLES LIKE 'order_snapshots';
SHOW COLUMNS FROM payments LIKE 'reference_hash';
SHOW COLUMNS FROM chat_messages LIKE 'signature';
SHOW TRIGGERS;
```

Restore the pre-upgrade backup into a fresh database, verify the restored migration records, and run the upgrade there with the original signing key and the configured data encryption key. Switch the API to that database only after the upgrade and health checks pass. Keep the failed database unchanged; if anything wrote to it after the backup, reconcile those writes before switching. Do not mark a partial migration successful or drop tables from the failed database to force a retry.



Use **Order integrity inspector** on the admin dashboard to inspect a full order ID. The server verifies the transaction chain and stored chat signatures. An unsigned transaction entry indicates a write outside the API and blocks further changes through the API until investigated. Orders and chat created before the migration have no historical proof. The chain detects modification of signed entries and gaps before later entries; a privileged database restore or truncation requires an independent backup or checkpoint to detect. GCash fingerprints reject reuse of a normalized reference. Staff still confirm the transfer directly, as a fingerprint cannot prove receipt.

New checkouts save one signed receipt in `order_snapshots` as part of the order transaction. The receipt locks the listing, vendor, weight, checkout price, payment method and quoted delivery charges. The customer and assigned rider can see this receipt; vendors can see only their own lines. The admin inspector checks the receipt against its signed audit entry. Orders placed before signed receipts were introduced retain their original checkout figures without a signature. The receipt always shows the checkout total. COD collection and vendor settlement continue to use the reconciled amounts displayed separately in the order.




The catalog starts empty. Register a vendor account, activate it in the admin dashboard, then sign in as the vendor and upload a product. Configure the R2 variables in the [backend guide](../crates/suki-backend/README.md) before uploading. Keep `SUKI_MEDIA_DIR` available and backed up for photos stored by earlier releases.





## Remembered sessions and device unlock

Generate and set a separate `SESSION_SIGNING_KEY` using `openssl rand -hex 32`. Share it with every API instance that serves this database and keep it in the server secret store. Apply migration 0009 before serving new login traffic. Existing opaque cookies remain valid until their original expiry; signing in again enrolls that browser or device into the new session system.

Access tokens last 15 minutes. Device sessions expire 30 days after sign-in. Refresh tokens rotate on use; a duplicate submitted within five seconds is rejected without issuing credentials, and later reuse revokes the affected device session. Sign out clears the device’s saved credentials after the server revokes them. You can revoke other devices from **Account → Sign-in and devices**. Signing in again is required after expiry, revocation or loss of access to secure storage.

Android and iOS remember the refresh credential in device-local secure storage. **Enable biometric login** asks the OS to protect access with biometrics or its device credential fallback. On Android 9–10, use your Suki password if fingerprint unlock is unavailable. After using a password to replace an inaccessible biometric vault, you can enable biometric login again. When enabled, the native app prompts on a fresh launch and before checkout or vendor payment profile changes. The browser uses HttpOnly cookies and does not display native biometric settings.

## Native mobile builds



Run the following commands from the workspace root. Use Dioxus CLI 0.7.10 to match the application. Android requires JDK 17, SDK platform 35, build-tools 35.0.0 and NDK 27.2.12479018; set `ANDROID_HOME` and `NDK_HOME`. iOS requires macOS and Xcode. Device builds need your signing configuration.


```sh
cargo install dioxus-cli --version 0.7.10 --locked
rustup target add aarch64-linux-android
export SUKI_API_URL=https://api.your-suki-host.example
cd crates/suki-mobile
dx build --platform android --target aarch64-linux-android --features mobile --locked
# On macOS, for an Apple Silicon simulator:
rustup target add aarch64-apple-ios-sim
dx build --platform ios --target aarch64-apple-ios-sim --features mobile --locked
```

For native builds, `SUKI_API_URL` is the API origin without a path. Set `SUKI_APP_ORIGIN` at build time only if the server uses a different `APP_ORIGIN` for WebSocket validation. Use HTTPS and secure cookies for a device. Device localhost refers to the device itself.

## Browser development preview


The same Rust UI can compile to WebAssembly for previewing the flows from a browser. Stop the API started above before launching the preview server. Run the following commands from the workspace root, with the API configured for `http://localhost:8080`:


```sh
rustup target add wasm32-unknown-unknown
cd crates/suki-mobile
dx build --platform web --features web --locked
cd ../..
SUKI_PREVIEW_DIR="$PWD/target/dx/suki-mobile/debug/web/public" cargo run --locked -p suki-backend
```

Open `http://localhost:8080`. The API serves the generated browser build, images and WebSocket endpoint together so cookie and origin rules still apply. Generated build files are ignored.

## Delivery, payments and operations


Set `GOOGLE_MAPS_API_KEY` on the API. New quotes use Google Routes driving distance from Bankerohan Market and the estimated basket weight: ₱85 minimum, ₱7/km, plus ₱5/kg. Checkout confirms that all items belong to one stall and revalidates the weight against the current product records. The existing goods minimum still applies.

Vendors configure a GCash mobile number in their dashboard. Customers transfer the order total to that number and submit a reference for manual review by the assigned rider or an administrator. For COD, the rider records the exact cash collected, including delivery, at handoff. The rider records the vendor settlement after delivery. The app records payments and settlements; it does not move money. The operator and vendor must agree how to remit the delivery portion and handle refunds for direct transfers.

Riders can enable location sharing on the active delivery screen. Leaving that screen stops updates. Android 11 and newer request fresh fixes; Android 9–10 can report fresh cached fixes or manual coordinates. iOS uses foreground Core Location. This directory does not include a production payment provider integration.


See [Backend operations](../crates/suki-backend/README.md#deployment-backups-and-operations) and [Architecture](ARCHITECTURE.md).


<