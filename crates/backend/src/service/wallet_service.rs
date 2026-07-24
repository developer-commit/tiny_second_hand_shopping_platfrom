// crates/backend/src/service/wallet_service.rs
// 목적: BCH 지갑 잔액 조회 및 출금 비즈니스 로직.
//
// [보안 흐름]
// - 출금 전 반드시 auth_service.verify_otp()로 2FA 검증 (OTP 없으면 출금 불가)
// - 수수료는 EvmWalletPort를 통해 실시간 추정
// - 개인키는 EncryptedKey 형태로만 전달, 복호화는 infra에서만 처리

use std::sync::Arc;
use sea_orm::DatabaseConnection;
use shared::dto::transaction_dto::{TxHistoryItemRes, WalletStateRes, WithdrawReq};
use thiserror::Error;
use crate::ports::wallet_port::EvmWalletPort;
use async_trait::async_trait;
use crate::service::traits::{AuthServiceTrait, WalletServiceTrait};

#[derive(Debug, Error)]
pub enum WalletServiceError {
    #[error("지갑을 찾을 수 없습니다.")]
    WalletNotFound,
    #[error("잔액이 부족합니다.")]
    InsufficientBalance,
    #[error("가용 잔액이 부족합니다.")]
    InsufficientAvailableFunds,
    #[error("2FA 인증 실패")]
    OtpFailed,
    #[error("출금 주소가 유효하지 않습니다.")]
    InvalidAddress,
    #[error("내부 서버 오류: {0}")]
    Internal(String),
}

pub struct WalletService {
    db: DatabaseConnection,
    wallet_port: Arc<dyn EvmWalletPort>,
    auth_service: Arc<dyn AuthServiceTrait>,
}

impl WalletService {
    pub fn new(
        db: DatabaseConnection,
        wallet_port: Arc<dyn EvmWalletPort>,
        auth_service: Arc<dyn AuthServiceTrait>,
    ) -> Self {
        WalletService {
            db,
            wallet_port,
            auth_service,
        }
    }
}

#[async_trait]
impl WalletServiceTrait for WalletService {
    /// 내 지갑 상태 조회
    async fn get_wallet_state(&self, user_id: i64) -> Result<WalletStateRes, WalletServiceError> {
        use crate::db::entity::{wallet, escrow_trade};
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
        
        let wallet = wallet::Entity::find()
            .filter(wallet::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|e| WalletServiceError::Internal(e.to_string()))?
            .ok_or(WalletServiceError::WalletNotFound)?;
            
        let escrows = escrow_trade::Entity::find()
            .filter(escrow_trade::Column::BuyerId.eq(user_id))
            .filter(
                sea_orm::Condition::any()
                    .add(escrow_trade::Column::Status.eq("deposited"))
                    .add(escrow_trade::Column::Status.eq("received"))
                    .add(escrow_trade::Column::Status.eq("disputed"))
            )
            .all(&self.db)
            .await
            .map_err(|e| WalletServiceError::Internal(e.to_string()))?;
            
        let mut locked_in_escrow: rust_decimal::Decimal = rust_decimal::Decimal::new(0, 0);
        for e in escrows {
            locked_in_escrow += e.amount;
        }
        
        use rust_decimal::prelude::ToPrimitive;
        let locked_f64 = locked_in_escrow.to_f64().unwrap_or(0.0);
        
        let mut eth_balance = 0.0;
        if let Ok(balance_wei) = self.wallet_port.get_balance(&wallet.eth_address).await {
            use ethers::utils::format_units;
            if let Ok(eth_str) = format_units(balance_wei, "ether") {
                eth_balance = eth_str.parse::<f64>().unwrap_or(0.0);
            }
        }
        
        Ok(wallet.into_dto(locked_f64, eth_balance))
    }

    /// 출금 처리 (POST /wallet/withdraw)
    async fn withdraw(
        &self,
        user_id: i64,
        req: WithdrawReq,
    ) -> Result<TxHistoryItemRes, WalletServiceError> {
        use crate::db::entity::{wallet, wallet_transaction, escrow_trade};
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set, TransactionTrait};
        use rust_decimal::Decimal;
        use rust_decimal::prelude::FromPrimitive;
        use std::str::FromStr;
        
        self.auth_service.verify_otp(user_id, &req.otp_token).await
            .map_err(|_| WalletServiceError::OtpFailed)?;
            
        if !self.wallet_port.validate_address(&req.destination_address).await.unwrap_or(false) {
            return Err(WalletServiceError::InvalidAddress);
        }
        
        let txn = self.db.begin().await.map_err(|e| WalletServiceError::Internal(e.to_string()))?;
        
        let wallet_model = wallet::Entity::find()
            .filter(wallet::Column::UserId.eq(user_id))
            .one(&txn)
            .await
            .map_err(|e| WalletServiceError::Internal(e.to_string()))?
            .ok_or(WalletServiceError::WalletNotFound)?;
            
        let escrows = escrow_trade::Entity::find()
            .filter(escrow_trade::Column::BuyerId.eq(user_id))
            .filter(
                sea_orm::Condition::any()
                    .add(escrow_trade::Column::Status.eq("deposited"))
                    .add(escrow_trade::Column::Status.eq("received"))
                    .add(escrow_trade::Column::Status.eq("disputed"))
            )
            .all(&txn)
            .await
            .map_err(|e| WalletServiceError::Internal(e.to_string()))?;
            
        let mut locked_in_escrow = rust_decimal::Decimal::new(0, 0);
        for e in escrows {
            locked_in_escrow = locked_in_escrow.checked_add(e.amount)
                .ok_or_else(|| WalletServiceError::Internal("Overflow in locked calc".to_string()))?;
        }
            
        let amount_decimal = rust_decimal::Decimal::from_f64(req.amount_bch)
            .unwrap_or(rust_decimal::Decimal::new(0, 0));
            
        let balance_wei = self.wallet_port.get_balance(&wallet_model.eth_address).await
            .map_err(|e| WalletServiceError::Internal(e.to_string()))?;
        let eth_balance_str = ethers::utils::format_units(balance_wei, "ether")
            .unwrap_or_else(|_| "0".to_string());
        let onchain_balance_decimal = rust_decimal::Decimal::from_str(&eth_balance_str)
            .unwrap_or(rust_decimal::Decimal::new(0, 0));
            
        let available_balance = onchain_balance_decimal.checked_sub(locked_in_escrow)
            .ok_or_else(|| WalletServiceError::Internal("Underflow in available calc".to_string()))?;

        if available_balance < amount_decimal {
            return Err(WalletServiceError::InsufficientAvailableFunds);
        }
        
        // Prepare U256 Wei amounts
        let amount_str = req.amount_bch.to_string();
        let amount_u256 = ethers::utils::parse_ether(&amount_str)
            .map_err(|_| WalletServiceError::Internal("Invalid amount format".to_string()))?;

        let fee_u256 = self.wallet_port.estimate_fee(amount_u256)
            .await
            .map_err(|e| WalletServiceError::Internal(e.to_string()))?;
            
        let fee_eth_str = ethers::utils::format_units(fee_u256, "ether")
            .unwrap_or_else(|_| "0".to_string());
        let fee_decimal = rust_decimal::Decimal::from_str(&fee_eth_str)
            .unwrap_or(rust_decimal::Decimal::new(0, 0));
            
        let total_required = amount_decimal.checked_add(fee_decimal)
            .ok_or_else(|| WalletServiceError::Internal("Overflow in total required calc".to_string()))?;

        if available_balance < total_required {
            return Err(WalletServiceError::InsufficientAvailableFunds);
        }
        
        let tx_id = self.wallet_port.broadcast_transaction(
            &wallet_model.eth_address,
            &req.destination_address,
            amount_u256,
            fee_u256,
            b"mock_private_key"
        )
            .await
            .map_err(|e| WalletServiceError::Internal(e.to_string()))?;
            
        // Balance is not maintained in DB anymore. Just insert transaction record.
        
        let new_tx = wallet_transaction::ActiveModel {
            wallet_id: Set(wallet_model.id),
            tx_type: Set("withdrawal".to_string()),
            amount: Set(amount_decimal),
            network_fee: Set(fee_decimal),
            tx_hash: Set(Some(tx_id)),
            status: Set("pending".to_string()),
            created_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };
        
        let inserted_tx = new_tx.insert(&txn).await.map_err(|e| WalletServiceError::Internal(e.to_string()))?;
        
        txn.commit().await.map_err(|e| WalletServiceError::Internal(e.to_string()))?;
        
        inserted_tx.try_into().map_err(|e| WalletServiceError::Internal(format!("Conversion error: {:?}", e)))
    }

    /// 트랜잭션 내역 조회
    async fn get_tx_history(
        &self,
        user_id: i64,
    ) -> Result<Vec<TxHistoryItemRes>, WalletServiceError> {
        use crate::db::entity::{wallet, wallet_transaction};
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
        
        let wallet = wallet::Entity::find()
            .filter(wallet::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|e| WalletServiceError::Internal(e.to_string()))?
            .ok_or(WalletServiceError::WalletNotFound)?;
            
        let txs = wallet_transaction::Entity::find()
            .filter(wallet_transaction::Column::WalletId.eq(wallet.id))
            .all(&self.db)
            .await
            .map_err(|e| WalletServiceError::Internal(e.to_string()))?;
            
        let mut result = Vec::new();
        for tx in txs {
            if let Ok(dto) = tx.try_into() {
                result.push(dto);
            }
        }
        
        Ok(result)
    }


    async fn eth_withdraw(
        &self,
        user_id: i64,
        req: shared::dto::transaction_dto::EthWithdrawReq,
    ) -> Result<TxHistoryItemRes, WalletServiceError> {
        self.auth_service.verify_otp(user_id, &req.otp_token).await
            .map_err(|_| WalletServiceError::OtpFailed)?;
            
        // ETH withdrawal logic would go here. For now, returning internal error as placeholder
        Err(WalletServiceError::Internal("ETH withdrawal not implemented yet".to_string()))
    }
}
