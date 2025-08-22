use common::{Address, Amount, Connector, Currency, Result, TxId, demo_id, GatewayError};

#[derive(Debug, Default)]
pub struct MockConnector;

impl Connector for MockConnector {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn generate_address(&self, currency: Currency) -> Result<Address> {
        Ok(Address {
            address: demo_id(match currency {
                Currency::BTC => "btc",
                Currency::ETH => "eth",
                Currency::SOL => "sol",
                Currency::SUI => "sui",
                Currency::XRP => "xrp",
            }),
            currency,
        })
    }

    fn quote_payment(&self, amount: Amount) -> Result<f64> {
        if amount.value <= 0.0 {
            return Err(GatewayError::InvalidAmount);
        }
        // Demo "fee": 0.2% + flat 0.0001
        Ok(amount.value * 0.002 + 0.0001)
    }

    fn broadcast_tx(&self, _currency: Currency, _signed_tx: &str) -> Result<TxId> {
        Ok(TxId(demo_id("tx")))
    }
}

pub fn build() -> MockConnector {
    MockConnector::default()
}
