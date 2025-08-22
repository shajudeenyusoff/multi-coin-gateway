use std::net::SocketAddr;
use std::sync::Arc;

use axum::{routing::{get, post}, Json, Router};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

// NEW: bring in TcpListener and use axum::serve (Axum 0.7 style)
use tokio::net::TcpListener;

use common::{Connector, Currency, Amount, Address, TxId};

use connector_mock as mock;

#[derive(Clone)]
struct AppState {
    connector: Arc<dyn Connector>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    setup_tracing();

    // Select connector (env CONNECTOR=mock for now)
    let connector_name = std::env::var("CONNECTOR").unwrap_or_else(|_| "mock".to_string());
    let connector: Arc<dyn Connector> = match connector_name.as_str() {
        "mock" | _ => Arc::new(mock::build()),
    };

    let state = AppState { connector };

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/quote", post(quote))
        .route("/v1/address/new", post(new_address))
        .route("/v1/tx/broadcast", post(broadcast))
        .with_state(state);

    let port: u16 = std::env::var("APP_PORT")
        .ok().and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 gateway-api listening on {}", addr);

    // Axum 0.7-compatible server bootstrap:
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn setup_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,axum=info,tower_http=info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();
}

async fn health() -> &'static str {
    "ok"
}

#[derive(Deserialize)]
struct QuoteReq {
    currency: Currency,
    amount: f64,
}

#[derive(Serialize)]
struct QuoteRes {
    currency: Currency,
    amount: f64,
    fee_estimate: f64,
    total: f64,
}

async fn quote(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(req): Json<QuoteReq>,
) -> axum::Json<QuoteRes> {
    let input = Amount { value: req.amount, currency: req.currency };
    // mock connector never errors here; if you plug real ones, handle Result properly
    let fee = state.connector.quote_payment(input).unwrap_or(0.0);
    let total = req.amount + fee;
    Json(QuoteRes { currency: req.currency, amount: req.amount, fee_estimate: fee, total })
}

#[derive(Deserialize)]
struct NewAddressReq {
    currency: Currency,
}

async fn new_address(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(req): Json<NewAddressReq>,
) -> axum::Json<Address> {
    let addr = state.connector.generate_address(req.currency)
        .unwrap_or(Address { address: "error".into(), currency: req.currency });
    Json(addr)
}

#[derive(Deserialize)]
struct BroadcastReq {
    currency: Currency,
    signed_tx: String,
}

#[derive(Serialize)]
struct BroadcastRes {
    tx_id: String,
}

async fn broadcast(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(req): Json<BroadcastReq>,
) -> axum::Json<BroadcastRes> {
    let txid = state.connector.broadcast_tx(req.currency, &req.signed_tx)
        .unwrap_or(TxId("error".into()));
    axum::Json(BroadcastRes { tx_id: txid.0 })
}
