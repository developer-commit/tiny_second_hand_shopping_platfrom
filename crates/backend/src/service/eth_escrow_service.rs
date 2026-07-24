use crate::ports::notification_port::NotificationPort;
use crate::service::traits::{EscrowServiceError, EscrowServiceTrait};
use crate::utils::security::obfuscate;
use async_trait::async_trait;
use ethers::prelude::*;
use sea_orm::DatabaseConnection;
use shared::dto::escrow_dto::{DisputeEscrowReq, InitiateEscrowReq, SafeTradeStatusRes, TradeStep};
use std::convert::TryFrom;
use std::sync::Arc;

// Generate bindings for the smart contract
abigen!(Escrow, "../../out/Escrow.sol/Escrow.json");

pub struct EthEscrowService {
    db: DatabaseConnection,
    notification_port: Arc<dyn NotificationPort>,
    blockchain_manager: Arc<crate::utils::blockchain::BlockchainManager>,
    wallet_port: Arc<dyn crate::ports::wallet_port::EvmWalletPort>,
    contract_address: Address,
}

impl EthEscrowService {
    pub fn new(
        db: DatabaseConnection,
        notification_port: Arc<dyn NotificationPort>,
        blockchain_manager: Arc<crate::utils::blockchain::BlockchainManager>,
        wallet_port: Arc<dyn crate::ports::wallet_port::EvmWalletPort>,
        contract_address: Address,
    ) -> Self {
        EthEscrowService {
            db,
            notification_port,
            blockchain_manager,
            wallet_port,
            contract_address,
        }
    }
}

#[async_trait]
impl EscrowServiceTrait for EthEscrowService {
    async fn initiate(
        &self,
        buyer_id: i64,
        req: InitiateEscrowReq,
    ) -> Result<SafeTradeStatusRes, EscrowServiceError> {
        use crate::db::entity::{escrow_trade, product, wallet};
        use crate::utils::security::deobfuscate;
        use sea_orm::{
            ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait,
        };

        let product_id =
            deobfuscate(&req.item_uid).map_err(|_| EscrowServiceError::ProductNotFound)?;
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?;

        let existing_trade = escrow_trade::Entity::find()
            .filter(escrow_trade::Column::ProductId.eq(product_id))
            .one(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?;

        if let Some(trade) = existing_trade {
            if trade.buyer_id != buyer_id && trade.seller_id != buyer_id {
                let _ = txn.rollback().await;
                return Err(EscrowServiceError::Forbidden);
            }
            let _ = txn.rollback().await;
            return trade
                .try_into()
                .map_err(|e| EscrowServiceError::Internal(format!("DTO: {:?}", e)));
        }

        let p = product::Entity::find_by_id(product_id)
            .one(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::ProductNotFound)?;

        if p.seller_id == buyer_id {
            let _ = txn.rollback().await;
            return Err(EscrowServiceError::EscrowNotInitiated);
        }

        if p.status != "on_sale" {
            let _ = txn.rollback().await;
            return Err(EscrowServiceError::InvalidState(p.status));
        }

        let _buyer_wallet = wallet::Entity::find()
            .filter(wallet::Column::UserId.eq(buyer_id))
            .one(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::Internal("Wallet not found".to_string()))?;

        let required_amount = p.price;

        let amount_str = required_amount.to_string();
        let required_amount_f64 =
            rust_decimal::prelude::ToPrimitive::to_f64(&required_amount).unwrap_or(0.0);

        // Check sufficient funds before initiation
        crate::service::wallet_domain::WalletDomain::verify_sufficient_funds(
            &txn,
            self.wallet_port.clone(),
            buyer_id,
            "ETH",
            required_amount_f64,
            0.0,
        )
        .await
        .map_err(|e| EscrowServiceError::InsufficientBalance)?;

        let auto_confirm = chrono::Utc::now() + chrono::Days::new(3);

        let escrow = escrow_trade::ActiveModel {
            product_id: Set(p.id),
            buyer_id: Set(buyer_id),
            seller_id: Set(p.seller_id),
            currency: Set("ETH".to_string()),
            amount: Set(required_amount),
            platform_fee: Set(rust_decimal::Decimal::new(0, 0)),
            status: Set("pending_deposit".to_string()),
            auto_confirm_at: Set(Some(auto_confirm.into())),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };

        let inserted_escrow = escrow
            .insert(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?;

        let amount_str = required_amount.to_string();
        let amount_u256 = ethers::utils::parse_ether(&amount_str).unwrap_or(U256::zero());

        let seller_address = wallet::Entity::find()
            .filter(wallet::Column::UserId.eq(p.seller_id))
            .one(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::Internal(
                "Seller wallet not found".to_string(),
            ))?
            .eth_address;

        let seller_addr: Address = seller_address
            .parse()
            .map_err(|_| EscrowServiceError::InvalidWalletAddress)?;

        let mut active_p: product::ActiveModel = p.into();
        active_p.status = Set("reserved".to_string());
        active_p.updated_at = Set(chrono::Utc::now().into());
        active_p
            .update(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?;

        txn.commit()
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?;

        let res: SafeTradeStatusRes = inserted_escrow
            .try_into()
            .map_err(|e| EscrowServiceError::Internal(format!("DTO: {:?}", e)))?;

        Ok(res)
    }

    async fn deposit(&self, buyer_id: i64, trade_id: i64) -> Result<(), EscrowServiceError> {
        use crate::db::entity::{escrow_trade, wallet};
        use sea_orm::{
            ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait,
        };

        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?;

        let trade = escrow_trade::Entity::find_by_id(trade_id)
            .one(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::TradeNotFound)?;

        if trade.buyer_id != buyer_id {
            return Err(EscrowServiceError::Forbidden);
        }

        if trade.status != "pending_deposit" {
            return Err(EscrowServiceError::InvalidState(trade.status));
        }

        let seller_wallet = wallet::Entity::find()
            .filter(wallet::Column::UserId.eq(trade.seller_id))
            .one(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::Internal(
                "Seller wallet not found".to_string(),
            ))?;

        let seller_addr: Address = seller_wallet
            .eth_address
            .parse()
            .map_err(|_| EscrowServiceError::Internal("Invalid seller address".to_string()))?;

        let amount_str = trade.amount.to_string();
        let amount_u256 = ethers::utils::parse_ether(&amount_str)
            .map_err(|_| EscrowServiceError::Internal("Invalid amount format".to_string()))?;

        // Check sufficient funds before deposit
        let required_amount_f64 =
            rust_decimal::prelude::ToPrimitive::to_f64(&trade.amount).unwrap_or(0.0);
        crate::service::wallet_domain::WalletDomain::verify_sufficient_funds(
            &txn,
            self.wallet_port.clone(),
            buyer_id,
            "ETH",
            required_amount_f64,
            0.0,
        )
        .await
        .map_err(|e| EscrowServiceError::InsufficientBalance)?;

        // INTERACT WITH SMART CONTRACT / EVM
        let dummy_privkey = std::env::var("MOCK_USER_PRIVATE_KEY").unwrap_or_else(|_| {
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()
        });
        let local_wallet = dummy_privkey
            .parse::<LocalWallet>()
            .map_err(|_| EscrowServiceError::Internal("Invalid private key".to_string()))?
            .with_chain_id(self.blockchain_manager.chain_id());
        let client = Arc::new(SignerMiddleware::new(
            self.blockchain_manager.provider().clone(),
            local_wallet,
        ));

        let contract = Escrow::new(self.contract_address, client);

        let call = contract
            .deposit(U256::from(trade_id), seller_addr)
            .value(amount_u256);
        let pending_tx = call
            .send()
            .await
            .map_err(|e| EscrowServiceError::RpcError(e.to_string()))?;
        let receipt = tokio::time::timeout(std::time::Duration::from_secs(30), pending_tx)
            .await
            .map_err(|_| EscrowServiceError::RpcError("Transaction receipt timeout".to_string()))?
            .map_err(|e| EscrowServiceError::RpcError(e.to_string()))?;

        let tx_hash_str = match receipt {
            Some(r) => {
                if r.status != Some(U64::from(1)) {
                    return Err(EscrowServiceError::ContractRevert(
                        "Transaction failed on-chain".to_string(),
                    ));
                }
                format!("{:?}", r.transaction_hash)
            }
            None => {
                return Err(EscrowServiceError::Internal(
                    "No transaction receipt".to_string(),
                ));
            }
        };

        let mut active_trade: escrow_trade::ActiveModel = trade.into();
        active_trade.status = Set("deposited".to_string());
        active_trade.blockchain_tx_hash = Set(Some(tx_hash_str));
        active_trade.updated_at = Set(chrono::Utc::now().into());
        active_trade
            .update(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?;

        txn.commit()
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn confirm(&self, buyer_id: i64, trade_id: i64) -> Result<(), EscrowServiceError> {
        use crate::db::entity::{escrow_trade, product, wallet, wallet_transaction};
        use sea_orm::{
            ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait,
        };

        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?;

        let trade = escrow_trade::Entity::find_by_id(trade_id)
            .one(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::TradeNotFound)?;

        if trade.buyer_id != buyer_id {
            return Err(EscrowServiceError::Forbidden);
        }

        if trade.status != "deposited" && trade.status != "received" {
            return Err(EscrowServiceError::InvalidState(trade.status));
        }

        let seller_wallet = wallet::Entity::find()
            .filter(wallet::Column::UserId.eq(trade.seller_id))
            .one(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::Internal(
                "Seller wallet not found".to_string(),
            ))?;

        let seller_addr: Address = seller_wallet
            .eth_address
            .parse()
            .map_err(|_| EscrowServiceError::Internal("Invalid seller address".to_string()))?;

        let amount_str = trade.amount.to_string();
        let amount_u256 = ethers::utils::parse_ether(&amount_str)
            .map_err(|_| EscrowServiceError::Internal("Invalid amount format".to_string()))?;

        // INTERACT WITH SMART CONTRACT / EVM
        let admin_privkey = std::env::var("MASTER_WALLET_KEY").unwrap_or_else(|_| {
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()
        });
        let admin_privkey = admin_privkey.trim_start_matches("0x");
        let local_wallet = admin_privkey
            .parse::<LocalWallet>()
            .map_err(|_| EscrowServiceError::Internal("Invalid admin private key".to_string()))?
            .with_chain_id(self.blockchain_manager.chain_id());
        let client = Arc::new(SignerMiddleware::new(
            self.blockchain_manager.provider().clone(),
            local_wallet,
        ));

        let contract = Escrow::new(self.contract_address, client);

        let call = contract.release(U256::from(trade_id));
        let pending_tx = call
            .send()
            .await
            .map_err(|e| EscrowServiceError::RpcError(e.to_string()))?;
        let receipt = tokio::time::timeout(std::time::Duration::from_secs(30), pending_tx)
            .await
            .map_err(|_| EscrowServiceError::RpcError("Transaction receipt timeout".to_string()))?
            .map_err(|e| EscrowServiceError::RpcError(e.to_string()))?;

        let tx_hash_str = match receipt {
            Some(r) => {
                if r.status != Some(U64::from(1)) {
                    return Err(EscrowServiceError::ContractRevert(
                        "Release transaction failed on-chain".to_string(),
                    ));
                }
                format!("{:?}", r.transaction_hash)
            }
            None => {
                return Err(EscrowServiceError::Internal(
                    "No transaction receipt".to_string(),
                ));
            }
        };

        let mut active_trade: escrow_trade::ActiveModel = trade.clone().into();
        active_trade.status = Set("settled".to_string());
        active_trade.updated_at = Set(chrono::Utc::now().into());
        active_trade
            .update(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?;

        let p = product::Entity::find_by_id(trade.product_id)
            .one(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::ProductNotFound)?;

        let mut active_p: product::ActiveModel = p.into();
        active_p.status = Set("sold".to_string());
        active_p.updated_at = Set(chrono::Utc::now().into());
        active_p
            .update(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?;

        // Persist transaction history
        let new_tx = wallet_transaction::ActiveModel {
            wallet_id: Set(seller_wallet.id),
            tx_type: Set("escrow_payout".to_string()),
            amount: Set(trade.amount),
            network_fee: Set(rust_decimal::Decimal::new(0, 0)),
            tx_hash: Set(Some(tx_hash_str)),
            status: Set("completed".to_string()),
            created_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };
        new_tx
            .insert(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?;

        txn.commit()
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn dispute(
        &self,
        buyer_id: i64,
        trade_id: i64,
        req: DisputeEscrowReq,
    ) -> Result<(), EscrowServiceError> {
        use crate::db::entity::escrow_trade;
        use sea_orm::{ActiveModelTrait, EntityTrait, Set};

        let trade = escrow_trade::Entity::find_by_id(trade_id)
            .one(&self.db)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::TradeNotFound)?;

        if trade.buyer_id != buyer_id {
            return Err(EscrowServiceError::Forbidden);
        }

        if trade.status != "deposited" && trade.status != "received" {
            return Err(EscrowServiceError::InvalidState(trade.status));
        }

        // INTERACT WITH SMART CONTRACT
        let admin_privkey = std::env::var("MASTER_WALLET_KEY").unwrap_or_else(|_| {
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()
        });
        let admin_privkey = admin_privkey.trim_start_matches("0x");
        let local_wallet = admin_privkey
            .parse::<LocalWallet>()
            .map_err(|_| EscrowServiceError::Internal("Invalid admin private key".to_string()))?
            .with_chain_id(self.blockchain_manager.chain_id());
        let client = Arc::new(SignerMiddleware::new(
            self.blockchain_manager.provider().clone(),
            local_wallet,
        ));
        let contract = Escrow::new(self.contract_address, client);

        let call = contract.dispute(U256::from(trade_id), req.cause.clone());
        let pending_tx = call
            .send()
            .await
            .map_err(|e| EscrowServiceError::RpcError(e.to_string()))?;
        let receipt = tokio::time::timeout(std::time::Duration::from_secs(30), pending_tx)
            .await
            .map_err(|_| EscrowServiceError::RpcError("Transaction receipt timeout".to_string()))?
            .map_err(|e| EscrowServiceError::ContractRevert(e.to_string()))?;

        match receipt {
            Some(r) => {
                if r.status != Some(U64::from(1)) {
                    return Err(EscrowServiceError::ContractRevert(
                        "Dispute transaction failed on-chain".to_string(),
                    ));
                }
            }
            None => {
                return Err(EscrowServiceError::Internal(
                    "No transaction receipt".to_string(),
                ));
            }
        }

        let mut active_trade: escrow_trade::ActiveModel = trade.clone().into();
        active_trade.status = Set("disputed".to_string());
        active_trade.dispute_reason = Set(Some(req.cause));
        active_trade.updated_at = Set(chrono::Utc::now().into());
        active_trade
            .update(&self.db)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn get_trade(
        &self,
        user_id: i64,
        trade_id: i64,
    ) -> Result<SafeTradeStatusRes, EscrowServiceError> {
        use crate::db::entity::escrow_trade;
        use sea_orm::EntityTrait;

        let trade = escrow_trade::Entity::find_by_id(trade_id)
            .one(&self.db)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::TradeNotFound)?;

        if trade.buyer_id != user_id && trade.seller_id != user_id {
            return Err(EscrowServiceError::Forbidden);
        }

        let res: SafeTradeStatusRes = trade
            .clone()
            .try_into()
            .map_err(|e| EscrowServiceError::Internal(format!("DTO: {:?}", e)))?;

        Ok(res)
    }

    async fn auto_confirm_expired_trades(&self) -> Result<(), EscrowServiceError> {
        Ok(())
    }
}
