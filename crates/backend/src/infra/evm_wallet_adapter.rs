// crates/backend/src/infra/evm_wallet_adapter.rs
use async_trait::async_trait;
use ethers::signers::{LocalWallet, Signer};
use crate::ports::wallet_port::{EvmWalletPort, NewWalletInfo, WalletPortError};

/// EVM 지갑 어댑터
pub struct EvmWalletAdapter {
    encryption_key: [u8; 32],
    api_base_url: String,
}

impl EvmWalletAdapter {
    pub fn new(encryption_key: [u8; 32], api_base_url: String) -> Self {
        EvmWalletAdapter { encryption_key, api_base_url }
    }
}

#[async_trait]
impl EvmWalletPort for EvmWalletAdapter {
    async fn create_wallet(&self, _user_id: i64) -> Result<NewWalletInfo, WalletPortError> {
        use crate::utils::security::encrypt_sensitive;
        
        let wallet = LocalWallet::new(&mut rand::thread_rng());
        let address = format!("{:#x}", wallet.address());
        
        let dummy_mnemonic_str = "legal winner thank year wave sausage worth useful recipe silver script ward".to_string();
        let enc_mnemonic = encrypt_sensitive(dummy_mnemonic_str.as_bytes(), &self.encryption_key)
            .map_err(|e| WalletPortError::CreateFailed(e.to_string()))?;
            
        let privkey_bytes = wallet.signer().to_bytes();
        let enc_priv = encrypt_sensitive(&privkey_bytes, &self.encryption_key)
            .map_err(|e| WalletPortError::CreateFailed(e.to_string()))?;

        Ok(NewWalletInfo {
            eth_address: address,
            encrypted_privkey: enc_priv.ciphertext().to_vec(),
            encrypted_mnemonic: enc_mnemonic.ciphertext().to_vec(),
        })
    }

    async fn validate_address(&self, address: &str) -> Result<bool, WalletPortError> {
        Ok(address.starts_with("0x") && address.len() == 42)
    }

    async fn estimate_fee(&self, _amount_wei: ethers::types::U256) -> Result<ethers::types::U256, WalletPortError> {
        Ok(ethers::types::U256::from(21000))
    }

    async fn broadcast_transaction(
        &self,
        _from_address: &str,
        _to_address: &str,
        _amount_wei: ethers::types::U256,
        _fee_wei: ethers::types::U256,
        _encrypted_privkey: &[u8],
    ) -> Result<String, WalletPortError> {
        Ok(format!("0x_tx_dummy_{}", chrono::Utc::now().timestamp()))
    }

    async fn get_balance(&self, address: &str) -> Result<ethers::types::U256, WalletPortError> {
        use ethers::providers::{Provider, Http, Middleware};
        use std::str::FromStr;
        let provider = Provider::<Http>::try_from(&self.api_base_url)
            .map_err(|e| WalletPortError::BalanceFetchFailed(e.to_string()))?;
        let addr = ethers::types::Address::from_str(address)
            .map_err(|_| WalletPortError::InvalidAddress)?;
        provider.get_balance(addr, None).await
            .map_err(|e| WalletPortError::BalanceFetchFailed(e.to_string()))
    }
}
