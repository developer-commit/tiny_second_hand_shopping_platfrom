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
        let url = format!("{}/address/{}/balance", self.api_base_url, address);
        let res = self.client.get(&url).send().await
            .map_err(|e| BchNetworkError::RequestFailed(e.to_string()))?;
        
        let body: serde_json::Value = res.json().await
            .map_err(|e| BchNetworkError::RequestFailed(e.to_string()))?;
            
        let confirmed = body.get("confirmed").and_then(|v| v.as_u64()).unwrap_or(0);
        let unconfirmed = body.get("unconfirmed").and_then(|v| v.as_u64()).unwrap_or(0);
        
        Ok(confirmed + unconfirmed)
    }

    async fn get_tx_confirmations(&self, tx_hash: &str) -> Result<u32, BchNetworkError> {
        let url = format!("{}/tx/{}", self.api_base_url, tx_hash);
        let res = self.client.get(&url).send().await
            .map_err(|e| BchNetworkError::RequestFailed(e.to_string()))?;
            
        let body: serde_json::Value = res.json().await
            .map_err(|e| BchNetworkError::RequestFailed(e.to_string()))?;
            
        if let Some(error) = body.get("error") {
            return Err(BchNetworkError::TxNotFound(error.to_string()));
        }
            
        Ok(body.get("confirmations").and_then(|v| v.as_u64()).unwrap_or(0) as u32)
    }

    async fn check_deposit(
        &self,
        platform_address: &str,
        expected_amount_satoshi: u64,
    ) -> Result<Option<String>, BchNetworkError> {
        let url = format!("{}/address/{}/txs", self.api_base_url, platform_address);
        let res = self.client.get(&url).send().await
            .map_err(|e| BchNetworkError::RequestFailed(e.to_string()))?;
            
        let txs: Vec<serde_json::Value> = res.json().await
            .map_err(|e| BchNetworkError::RequestFailed(e.to_string()))?;
            
        for tx in txs {
            if let Some(vout) = tx.get("vout").and_then(|v| v.as_array()) {
                for output in vout {
                    if let Some(script_pubkey) = output.get("scriptPubKey") {
                        if let Some(addresses) = script_pubkey.get("addresses").and_then(|a| a.as_array()) {
                            let contains_addr = addresses.iter().any(|a| a.as_str() == Some(platform_address));
                            // Ensure conversion to u64 gracefully, using as_f64 as API might return sats or bch
                            // Here we assume it returns satoshis, per the dummy implementation.
                            let value = output.get("value").and_then(|v| v.as_u64()).unwrap_or(0);
                            if contains_addr && value == expected_amount_satoshi {
                                if let Some(txid) = tx.get("txid").and_then(|t| t.as_str()) {
                                    return Ok(Some(txid.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }
}
