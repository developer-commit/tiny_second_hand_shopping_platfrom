// crates/backend/src/infra/bch_network_adapter.rs
// 목적: BchNetworkPort 구현체 — reqwest를 통한 Electrum HTTP 게이트웨이 통신.

use async_trait::async_trait;
use crate::ports::bch_network_port::{BchNetworkError, BchNetworkPort};

pub struct BchNetworkAdapter {
    client: reqwest::Client,
    api_base_url: String,
}

impl BchNetworkAdapter {
    pub fn new(api_base_url: String) -> Self {
        BchNetworkAdapter {
            client: reqwest::Client::new(),
            api_base_url,
        }
    }
}

#[async_trait]
impl BchNetworkPort for BchNetworkAdapter {
    async fn get_balance(&self, address: &str) -> Result<u64, BchNetworkError> {
        todo!("GET <api_base_url>/address/<address>/balance → satoshi 파싱")
    }

    async fn get_tx_confirmations(&self, tx_hash: &str) -> Result<u32, BchNetworkError> {
        todo!("GET <api_base_url>/tx/<tx_hash> → confirmations 필드 파싱")
    }

    async fn check_deposit(
        &self,
        platform_address: &str,
        expected_amount_satoshi: u64,
    ) -> Result<Option<String>, BchNetworkError> {
        todo!("GET <api_base_url>/address/<platform_address>/txs → 금액 매칭 확인 → tx_hash 반환")
    }
}
