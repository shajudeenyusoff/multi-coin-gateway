use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    BTC,
    ETH,
    SOL,
    SUI,
    XRP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Amount {
    pub value: f64,
    pub currency: Currency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub address: String,
    pub currency: Currency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxId(pub String);

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("unsupported currency: {0:?}")]
    UnsupportedCurrency(Currency),
    #[error("connector not available")]
    ConnectorUnavailable,
    #[error("invalid amount")]
    InvalidAmount,
    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, GatewayError>;

/// A minimal set of operations the gateway needs from each chain connector.
#[allow(unused_variables)]
pub trait Connector: Send + Sync + 'static {
    fn name(&self) -> &'static str;

    /// Generate a new receiving address (or payment request) for the given currency.
    fn generate_address(&self, currency: Currency) -> Result<Address>;

    /// Quote a payment (e.g., fee estimate, final total). This is intentionally simple.
    fn quote_payment(&self, amount: Amount) -> Result<f64>;

    /// Broadcast a signed transaction (string here for portability).
    fn broadcast_tx(&self, currency: Currency, signed_tx: &str) -> Result<TxId>;
}

/// Utility to create a pseudo-unique demo address or tx id.
pub fn demo_id(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4().as_simple())
}
