use std::sync::Arc;
use std::time::Instant;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::{info, Level};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

use common::{Address, ClientId, Currency};
use connector_mock::MockConnector; // NOTE: module name uses underscore
use gateway_core::{FeeConfig, FeeEngine, Gateway, Registry};

#[derive(Clone)]
struct AppState {
    gw: Arc<Gateway>,
    started_at: Instant,
    version: &'static str,
}

#[derive(Deserialize, ToSchema)]
struct CreateInvoiceReq {
    client_id: String,
    currency: Currency,
    amount: f64,
}

#[derive(Serialize, ToSchema)]
struct CreateInvoiceRes {
    address: Address,
    invoice_id: String,
    fee_percent: f64,
    fee_amount: f64,
    total_payable: f64,
}

#[utoipa::path(
    post,
    path = "/v1/invoices",
    request_body = CreateInvoiceReq,
    responses(
        (status = 200, description = "Invoice created", body = CreateInvoiceRes)
    )
)]
async fn create_invoice(
    State(st): State<AppState>,
    Json(req): Json<CreateInvoiceReq>,
) -> Json<CreateInvoiceRes> {
    let client = ClientId(req.client_id);
    let (address, invoice_id, fee) = st
        .gw
        .create_invoice(client, req.currency, req.amount)
        .await
        .expect("create_invoice");

    let total = req.amount + fee.fee_amount;
    Json(CreateInvoiceRes {
        address,
        invoice_id,
        fee_percent: fee.percent,
        fee_amount: fee.fee_amount,
        total_payable: total,
    })
}

#[derive(Deserialize, ToSchema)]
struct FeePreviewReq {
    client_id: String,
    amount: f64,
}

#[derive(Serialize, ToSchema)]
struct FeePreviewRes {
    fee_percent: f64,
    fee_amount: f64,
    total_with_fee: f64,
}

#[utoipa::path(
    post,
    path = "/v1/fees/preview",
    request_body = FeePreviewReq,
    responses((status = 200, description = "Fee preview", body = FeePreviewRes))
)]
async fn fee_preview(
    State(st): State<AppState>,
    Json(req): Json<FeePreviewReq>,
) -> Json<FeePreviewRes> {
    let client = ClientId(req.client_id);
    let fee = st.gw.fees().fee_for(&client, req.amount);
    Json(FeePreviewRes {
        fee_percent: fee.percent,
        fee_amount: fee.fee_amount,
        total_with_fee: req.amount + fee.fee_amount,
    })
}

#[derive(Serialize, ToSchema)]
struct HealthRes {
    service: &'static str,
    online: bool,
    uptime: String,
    uptime_seconds: u64,
    version: String,
}

#[utoipa::path(get, path = "/health", responses((status = 200, body = HealthRes)))]
async fn health(State(st): State<AppState>) -> Json<HealthRes> {
    let secs = st.started_at.elapsed().as_secs();
    let uptime = format!("{}h {}m {}s", secs / 3600, (secs % 3600) / 60, secs % 60);
    Json(HealthRes {
        service: "Daraj Gateway",
        online: true,
        uptime,
        uptime_seconds: secs,
        version: st.version.to_string(),
    })
}

#[derive(OpenApi)]
#[openapi(
    paths(create_invoice, fee_preview, health),
    components(
        schemas(
            CreateInvoiceReq,
            CreateInvoiceRes,
            FeePreviewReq,
            FeePreviewRes,
            HealthRes,
            Address,
            Currency
        )
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    // Load .env (optional, harmless if missing)
    let _ = dotenvy::dotenv();

    // logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))
        .init();

    // Registry with 5 mock connectors (BTC/ETH/SOL/SUI/XRP)
    let reg = Registry::new()
        .with(Arc::new(MockConnector { cur: Currency::BTC }))
        .with(Arc::new(MockConnector { cur: Currency::ETH }))
        .with(Arc::new(MockConnector { cur: Currency::SOL }))
        .with(Arc::new(MockConnector { cur: Currency::SUI }))
        .with(Arc::new(MockConnector { cur: Currency::XRP }));

    // Fee tiers from ENV: FEE_TIERS="0:0.005,100:0.004,1000:0.003,10000:0.002,100000:0.001"
    let fee_cfg = FeeConfig::from_env();
    let fee_engine = FeeEngine::new(fee_cfg);

    let gw = Arc::new(Gateway::new(reg, fee_engine));
    let state = AppState {
        gw,
        started_at: Instant::now(),
        version: env!("CARGO_PKG_VERSION"),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/invoices", post(create_invoice))
        .route("/v1/fees/preview", post(fee_preview))
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .with_state(state);

    let port: u16 = std::env::var("HTTP_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    info!("→ listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
