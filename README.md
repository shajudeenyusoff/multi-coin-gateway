# Multi-Coin Gateway (Rust Workspace)

This is a production-ready **starter** for a multi-coin payment gateway in Rust.

## Crates
- `common` — shared types, errors, and the `Connector` trait.
- `connector-mock` — a mock connector that simulates blockchain operations.
- `gateway-api` — Axum HTTP API server with health and stub payment endpoints.

## Quick Start

### 1) Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# then restart your terminal or run:
source $HOME/.cargo/env
```

### 2) Run the API
```bash
cd crates/gateway-api
cargo run
# Server listens on 0.0.0.0:8080 by default
```

### 3) Try the endpoints
```bash
curl -s http://127.0.0.1:8080/health
curl -s http://127.0.0.1:8080/v1/quote -H 'content-type: application/json' -d '{"currency":"ETH","amount":1.25}'
curl -s http://127.0.0.1:8080/v1/address/new -H 'content-type: application/json' -d '{"currency":"BTC"}'
```

### 4) Configure
Create a `.env` file in `crates/gateway-api/`:
```
APP_PORT=8080
CONNECTOR=mock
```

### 5) Extend to real chains
Add new crates (e.g., `connector-eth`, `connector-btc`) implementing the `Connector` trait from `common`. In `gateway-api`, swap the selected connector via the `CONNECTOR` env var or dependency injection.

---

## Design
- **Trait-based connectors** so each chain is a plugin (BTC, ETH, SOL, SUI, XRP).
- **Axum + Tokio** for async, scalable API.
- **Tracing** for structured logs.
- **dotenvy** for config.

This skeleton compiles without external chain deps. You can progressively add real clients (e.g., `ethers` for ETH) later without blocking the API development.
