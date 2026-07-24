use ethers::prelude::*;
use eyre::Result;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub alias: String,
    pub address: String,
    pub private_key: String,
    #[serde(skip)]
    pub balance: U256,
}

impl UserAccount {
    pub fn new(alias: String) -> Self {
        let wallet = LocalWallet::new(&mut thread_rng());
        Self {
            alias,
            address: format!("{:?}", wallet.address()),
            private_key: hex::encode(wallet.signer().to_bytes()),
            balance: U256::zero(),
        }
    }
}

pub struct BlockchainManager {
    provider: Provider<Http>,
    master_wallet: LocalWallet,
    chain_id: u64,
}

impl BlockchainManager {
    pub async fn new(rpc_url: &str, master_key: &str) -> Result<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let chain_id = provider.get_chainid().await?.as_u64();
        let master_wallet = master_key.parse::<LocalWallet>()?.with_chain_id(chain_id);

        Ok(Self {
            provider,
            master_wallet,
            chain_id,
        })
    }

    pub fn provider(&self) -> &Provider<Http> {
        &self.provider
    }

    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }

    pub fn master_address(&self) -> Address {
        self.master_wallet.address()
    }

    pub async fn get_balance(&self, address: Address) -> Result<U256> {
        let balance = self.provider.get_balance(address, None).await?;
        Ok(balance)
    }

    pub async fn fund_account(&self, target: Address, amount: U256) -> Result<TxHash> {
        // Try Anvil cheatcode first
        let is_anvil = {
            let res: Result<U256, _> = self.provider.request("anvil_nodeInfo", ()).await;
            res.is_ok() || self.chain_id == 31337 // fallback heuristic
        };

        if is_anvil {
            // Use anvil_setBalance cheatcode
            let current_balance = self.get_balance(target).await?;
            let new_balance = current_balance.saturating_add(amount);
            let hex_balance = format!("{:#x}", new_balance);
            let address_hex = format!("{:?}", target);

            let _: () = self
                .provider
                .request("anvil_setBalance", (address_hex, hex_balance))
                .await?;
            Ok(TxHash::zero()) // No real tx hash for cheatcode
        } else {
            // Real transaction from master
            let client = SignerMiddleware::new(self.provider.clone(), self.master_wallet.clone());
            let tx = TransactionRequest::new().to(target).value(amount);

            let pending_tx = client.send_transaction(tx, None).await?;
            let receipt = pending_tx.await?.ok_or_else(|| eyre::eyre!("Tx dropped"))?;
            Ok(receipt.transaction_hash)
        }
    }

    pub async fn send_transaction(
        &self,
        from_key: &str,
        target: Address,
        amount: U256,
    ) -> Result<TxHash> {
        let wallet = from_key
            .parse::<LocalWallet>()?
            .with_chain_id(self.chain_id);
        let client = SignerMiddleware::new(self.provider.clone(), wallet);
        let tx = TransactionRequest::new().to(target).value(amount);

        let pending_tx = client.send_transaction(tx, None).await?;
        let receipt = pending_tx.await?.ok_or_else(|| eyre::eyre!("Tx dropped"))?;
        Ok(receipt.transaction_hash)
    }
}
