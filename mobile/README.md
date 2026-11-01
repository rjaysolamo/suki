<!-- coderabbit-ai-generated:v1 -->

# Suki Davao Android foundation

This is the first Phase 1 feasibility build of the approved Rust mobile migration. It is an installed Dioxus Mobile application with bundled UI and an isolated JNI runtime probe. It has no account, product, order or payment functionality yet.

## Build prerequisites

- Rust 1.99.1 (root rust-toolchain.toml), rustfmt and Clippy.
- Matching Dioxus CLI 0.7.10 and Dioxus crate 0.7.10.
- JDK 17; Android SDK platform 35 and build-tools 35.0.0.
- Android NDK 27.2.12479018; set ANDROID_HOME and NDK_HOME.
- Rust Android targets for the selected ABI. The app requires Android API 28 or newer.

The backend is intentionally excluded from this workspace while its PostgreSQL migration is pending. The existing web storefront and MariaDB backend remain legacy code and are not the new mobile product.

## Commands

From the repository root:

```sh
rustup target add aarch64-linux-android x86_64-linux-android
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cd mobile
dx build --platform android --target aarch64-linux-android --locked
```

Host compilation checks UI syntax but excludes the Android JNI implementation. Cargo test currently discovers zero tests; runtime behavior must be checked on Android. APK packaging and device verification are independent checks.

In the task sandbox, source `$HOME/runtime/android/suki-env.sh` before these commands to use the installed tools. This environment file is task-local, outside the repository. Java builds in this proxy environment require the installed sandbox CA in the JDK trust store; do not disable TLS validation.

## Native boundary

`src/platform.rs` is the only native integration in this feasibility build. It attaches to the Android VM supplied by ndk-context and reads Build.VERSION.SDK_INT. Errors are shown as retryable UI text. Future Google identity, Keystore-backed credential storage and document selection belong behind this boundary with small dedicated Kotlin adapters.

No credentials are collected or stored by this build. This runtime probe does not establish that Google sign-in, secure storage or Android lifecycle integration has passed.
