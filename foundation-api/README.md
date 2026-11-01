<!-- coderabbit-ai-generated:v1 -->

# Suki Phase 1 API foundation

This crate is the start of the PostgreSQL migration. It exposes liveness and database readiness endpoints only. The previous MariaDB backend is not served by this crate; customer login, sessions, applications, document upload, administrator approval and all marketplace operations remain unimplemented. Do not deploy this foundation as a working marketplace.

## Development

Set `DATABASE_URL` to a PostgreSQL database dedicated to this foundation. The runtime applies the migrations in `migrations/` at startup; use a database role with the necessary migration permissions. Do not point it at the legacy MariaDB database. Applied migration files must not be edited. The SQLx schema includes Google identity subjects, role grants, session hashes, vendor/courier application records, document object keys and audit records. These tables alone do not implement authentication, privacy, validation or authorization.

```sh
cargo run --locked -p suki-foundation-api
curl --fail http://127.0.0.1:8080/health/live
curl --fail http://127.0.0.1:8080/health/ready
```

The API binds to 127.0.0.1:8080 unless `BIND_ADDR` is set. The service must be behind HTTPS when deployed. Do not log connection strings or credentials.

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
SUKI_TEST_DATABASE_URL=postgres://.../suki_foundation_test cargo test --locked -p suki-foundation-api --test postgres -- --ignored
```

The ignored integration test requires a dedicated disposable database whose name ends in `_test`; it applies migrations and rolls back synthetic fixtures. <!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:a4e0d0b2ccd33a672dd80da73463dcc348b02eaa92837fe2648f0976b8a24c1a -->

The pricing function is a domain calculation only and requires a separately verified road distance. It is not a checkout implementation.

`identity::verify_google_id_token` validates a signed Google identity token using an already trusted public key and an expected nonce. The module does not fetch Google keys, issue or consume nonces, authenticate HTTP requests, issue sessions, or enable sign-in. A future caller must retrieve keys only from Google over HTTPS and enforce one-use, server-bound nonces before creating sessions. The test RSA key is synthetic and public.

<!-- /coderabbit-ai-generated:v1 -->
