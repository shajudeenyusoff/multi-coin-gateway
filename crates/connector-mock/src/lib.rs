use async_trait::async_trait;
use common::{Connector, Currency, Amount, Address, TxId, TxStatus, GatewayError};

#[derive(Clone, Default)]
pub struct MockConnector { pub cur: Currency }

#[async_trait]
impl Connector for MockConnector {
    fn currency(&self) -> Currency { self.cur }

    async fn validate_address(&self, addr: &str) -> Result<bool, GatewayError> {
        Ok(addr.starts_with("mock_"))
    }

    async fn new_deposit_address(&self) -> Result<Address, GatewayError> {
        Ok(Address { address: format!("mock_{}", uuid::Uuid::new_v4()), currency: self.cur })
    }

    async fn create_payment_request(&self, amount: Amount) -> Result<(Address, String), GatewayError> {
        let addr = self.new_deposit_address().await?;
        Ok((addr, uuid::Uuid::new_v4().to_string()))
    }

    async fn tx_status(&self, _tx: &TxId) -> Result<TxStatus, GatewayError> {
        Ok(TxStatus::Confirmed(1))
    }

    async fn balance(&self, addr: &Address) -> Result<Amount, GatewayError> {
        Ok(Amount { value: if addr.address.contains("zero") { 0.0 } else { 1.2345 }, currency: self.cur })
    }

    async fn send(&self, _from: &str, _to: &Address, amount: Amount) -> Result<TxId, GatewayError> {
        if amount.value <= 0.0 { return Err(GatewayError::Unknown("amount must be > 0".into())); }
        Ok(TxId(uuid::Uuid::new_v4().to_string()))
    }
}
