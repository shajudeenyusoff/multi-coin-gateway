use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency { BTC, ETH, SOL, SUI, XRP }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Amount { pub value: f64, pub currency: Currency }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address { pub address: String, pub currency: Currency }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxId(pub String);

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("invalid address: {0}")] InvalidAddress(String),
    #[error("network error: {0}")]  Network(String),
    #[error("not implemented")]     NotImplemented,
    #[error("unknown: {0}")]        Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TxStatus { Pending, Confirmed(u32), Failed(String) }

/// Logical client identifier (e.g., merchant account id)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClientId(pub String);

/// What fee was applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedFee {
    /// decimal fraction (e.g., 0.005 for 0.5%)
    pub percent: f64,
    /// fee amount in the transaction currency
    pub fee_amount: f64,
}

#[async_trait]
pub trait Connector: Send + Sync {
    fn currency(&self) -> Currency;
    async fn validate_address(&self, addr: &str) -> Result<bool, GatewayError>;
    async fn new_deposit_address(&self) -> Result<Address, GatewayError>;
    async fn create_payment_request(&self, amount: Amount) -> Result<(Address, String), GatewayError>; // (address, invoice_id)
    async fn tx_status(&self, tx: &TxId) -> Result<TxStatus, GatewayError>;
    async fn balance(&self, addr: &Address) -> Result<Amount, GatewayError>;
    async fn send(&self, from: &str, to: &Address, amount: Amount) -> Result<TxId, GatewayError>;
}
