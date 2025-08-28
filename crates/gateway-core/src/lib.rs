use std::{collections::HashMap, sync::Arc, time::{Duration, SystemTime}};
use common::{Connector, Currency, Amount, Address, TxId, TxStatus, GatewayError, ClientId, AppliedFee};

#[derive(Clone)]
pub struct FeeTier {
    pub min_tx_count_30d: u64,
    pub percent: f64,
}

#[derive(Clone)]
pub struct FeeConfig { pub tiers: Vec<FeeTier> }

impl FeeConfig {
    pub fn defaults() -> Self {
        let tiers = vec![
            FeeTier { min_tx_count_30d: 0,      percent: 0.005 },
            FeeTier { min_tx_count_30d: 100,    percent: 0.004 },
            FeeTier { min_tx_count_30d: 1_000,  percent: 0.003 },
            FeeTier { min_tx_count_30d: 10_000, percent: 0.002 },
            FeeTier { min_tx_count_30d: 100_000,percent: 0.001 },
        ];
        Self { tiers }
    }
    pub fn from_env() -> Self {
        if let Ok(s) = std::env::var("FEE_TIERS") {
            let mut tiers = Vec::new();
            for part in s.split(',') {
                let mut it = part.split(':');
                if let (Some(min_s), Some(p_s)) = (it.next(), it.next()) {
                    if let (Ok(min), Ok(p)) = (min_s.trim().parse::<u64>(), p_s.trim().parse::<f64>()) {
                        tiers.push(FeeTier { min_tx_count_30d: min, percent: p });
                    }
                }
            }
            if !tiers.is_empty() {
                tiers.sort_by_key(|t| t.min_tx_count_30d);
                return Self { tiers };
            }
        }
        Self::defaults()
    }
}

#[derive(Clone)]
pub struct FeeEngine {
    cfg: FeeConfig,
    counts_30d: Arc<parking_lot::RwLock<HashMap<ClientId, Vec<SystemTime>>>>,
}

impl FeeEngine {
    pub fn new(cfg: FeeConfig) -> Self {
        Self { cfg, counts_30d: Arc::new(parking_lot::RwLock::new(HashMap::new())) }
    }
    pub fn record_tx(&self, client: &ClientId) {
        let mut map = self.counts_30d.write();
        let entries = map.entry(client.clone()).or_default();
        entries.push(SystemTime::now());
        Self::prune_older_than(entries, Duration::from_secs(30*24*3600));
    }
    fn prune_older_than(v: &mut Vec<SystemTime>, win: Duration) {
        let cutoff = SystemTime::now() - win;
        v.retain(|&t| t >= cutoff);
    }
    pub fn current_count_30d(&self, client: &ClientId) -> u64 {
        let mut map = self.counts_30d.write();
        let entries = map.entry(client.clone()).or_default();
        Self::prune_older_than(entries, Duration::from_secs(30*24*3600));
        entries.len() as u64
    }
    pub fn fee_for(&self, client: &ClientId, amount_value: f64) -> AppliedFee {
        let count = self.current_count_30d(client);
        let mut chosen = self.cfg.tiers.first().expect("tiers");
        for t in &self.cfg.tiers {
            if count >= t.min_tx_count_30d { chosen = t; } else { break; }
        }
        let fee_amt = (amount_value * chosen.percent).max(0.0);
        AppliedFee { percent: chosen.percent, fee_amount: fee_amt }
    }
}

#[derive(Clone, Default)]
pub struct Registry { inner: HashMap<Currency, Arc<dyn Connector>> }

impl Registry {
    pub fn new() -> Self { Self { inner: HashMap::new() } }
    pub fn with(mut self, c: Arc<dyn Connector>) -> Self { self.inner.insert(c.currency(), c); self }
    pub fn get(&self, cur: Currency) -> Result<Arc<dyn Connector>, GatewayError> {
        self.inner.get(&cur).cloned().ok_or_else(|| GatewayError::Unknown(format!("no connector for {:?}", cur)))
    }
}

pub struct Gateway { reg: Registry, fee: FeeEngine }

impl Gateway {
    pub fn new(reg: Registry, fee: FeeEngine) -> Self { Self { reg, fee } }
    pub fn fees(&self) -> &FeeEngine { &self.fee }
    pub async fn create_invoice(&self, client: ClientId, cur: Currency, amount: f64)
        -> Result<(Address, String, AppliedFee), GatewayError> {
        let connector = self.reg.get(cur)?;
        let fee = self.fee.fee_for(&client, amount);
        self.fee.record_tx(&client);
        let (addr, invoice_id) = connector.create_payment_request(Amount{ value: amount, currency: cur }).await?;
        Ok((addr, invoice_id, fee))
    }
    pub async fn check_tx(&self, cur: Currency, tx: &TxId) -> Result<TxStatus, GatewayError> {
        self.reg.get(cur)?.tx_status(tx).await
    }
    pub async fn balance(&self, addr: &Address) -> Result<Amount, GatewayError> {
        self.reg.get(addr.currency)?.balance(addr).await
    }
}
