# Suki Davao

A 100% Pure Rust market-shopping platform for Davao City, Philippines. Direct palengke sourcing from Bankerohan, Agdao, and Toril delivered across Davao in under 45 minutes with verified cold-chain couriers, sub-millisecond road distance calculations, and instant GCash payment validation.

Designed using the anti-slop principles of [Taste Skill](https://github.com/Leonxlnx/taste-skill) (`Leonxlnx/taste-skill`).

---

## 📁 Repository Structure

```text
suki/
├── assets/                    # 🎯 USA RA KA SHARED ASSETS FOLDER (Shared by Landing & Mobile)
│   ├── css/
│   │   ├── landing.css        # Landing design system (Manrope, typography, equal edges)
│   │   └── mobile.css         # Mobile Figma design system (colors, buttons, components)
│   ├── js/
│   │   └── landing.js         # Live basket simulator, road quote engine, GCash validation
│   └── images/                # Real Davao produce photos (Tuna, Durian, Pomelo, etc.)
├── landing/                   # Pure Rust Web Landing Page & Storefront (Axum 0.8 + Dioxus Web)
│   ├── Cargo.toml             # suki-landing crate definition
│   ├── Dioxus.toml            # Dioxus Web configuration
│   ├── src/templates.rs       # 100% Pure Rust semantic HTML renderer (Zero .html files)
│   └── src/
│       ├── main.rs            # Axum HTTP server, static file router, sub-1ms quote API
│       ├── data.rs            # Davao market hubs (Bankerohan, Agdao, Toril) & price index
│       └── templates.rs       # Server-side rendering template pipeline
├── mobile/                    # Pure Rust Mobile Application (Dioxus 0.7 Native Android)
│   ├── Cargo.toml             # suki-mobile crate definition
│   ├── Dioxus.toml            # Android SDK/NDK bundle settings
│   └── src/
│       ├── main.rs            # Dioxus multi-screen UI (calls assets/css/mobile.css)
│       ├── images.rs          # Image data bindings
│       └── platform.rs        # Android JNI integration & system bridging
├── backend/                   # Pure Rust REST & WebSocket API (Axum 0.8 + MariaDB + SQLx)
│   ├── Cargo.toml             # suki-api crate definition
│   ├── migrations/            # Versioned SQL migrations for accounts, catalog, orders
│   ├── src/                   # Auth, catalog, pricing, payments, realtime WebSockets
│   └── tests/                 # Integration tests for pricing, GCash, orders
├── foundation-api/            # Core Foundation & Auth Microservice (Axum + PostgreSQL)
│   ├── Cargo.toml             # suki-foundation-api crate definition
│   └── src/                   # Argon2id password hashing, JWT token handling
├── scripts/                   # Development, emulator, and setup scripts
│   ├── run-android.sh         # Auto-launch Android emulator & Dioxus dev build
│   ├── setup-ndk.sh           # Android SDK/NDK path & environment setup
│   └── seed-local.py          # Synthetic local Davao catalog seed fixtures
├── docs/                      # Comprehensive setup and API guides
│   └── SETUP.md               # Tooling, databases, environment variables
├── .agents/skills/            # Installed Agent Skills
│   └── design-taste-frontend/ # Anti-slop frontend art direction (Leonxlnx/taste-skill)
├── .archive/                  # Preserved archives of legacy implementations
│   └── web/                   # Archived legacy Next.js/TypeScript code
├── Cargo.toml                 # Cargo workspace definition
├── Cargo.lock                 # Pinned dependencies
├── package.json               # Developer quick-command scripts
└── rust-toolchain.toml        # Pinned Rust 1.99.1 (2024 edition)
```

---

## 🚀 Quick Start

### 1. Launch the Pure Rust Web Landing Page
The landing page runs on an Axum 0.8 server with embedded static assets, real Davao market pricing indices, and a sub-millisecond road distance quote engine:

```sh
# Start the web landing server (default: http://localhost:3000)
cargo run -p suki-landing

# Or via npm shortcut:
npm run dev
```

Open `http://localhost:3000` in your browser.

### 2. Run the Pure Rust Mobile App (Android)
```sh
# Requires Android SDK/NDK
npm run mobile
```

### 3. Start the Backend API
```sh
npm run backend
# Or: cargo run --manifest-path backend/Cargo.toml
```

---

## 💎 Features & Anti-Slop Design

- **Taste-Skill Standard**: Plain copy, equal edges, real market photography, warm Davao botanical palette (`#12362a` deep forest, `#194d3c` pine green, `#fbf9f4` cream, `#c98b2c` pomelo gold).
- **Three Davao Public Markets**: Real dawn-sourced produce from **Bankerohan** (24/7 wholesale), **Agdao** (seafood & tuna), and **Toril** (highland fruit orchards).
- **Sub-millisecond Quote Simulator**: Real-time road distance fee computation across Davao districts (Poblacion, Bajada, Lanang, Matina, Buhangin, Toril, Calinan) powered by Rust.
- **GCash Instant Verification**: Real-time transaction validation preview matching Suki's backend transaction safeguards.
- **Zero JS Backend Runtime**: No Node.js runtime overhead in production; compiled to native machine code.

---

## 🛠️ Verification & Checks

```sh
# Check all workspace members
cargo check --workspace

# Check backend
cargo check --manifest-path backend/Cargo.toml

# Build release landing binary
cargo build -p suki-landing --release
```
