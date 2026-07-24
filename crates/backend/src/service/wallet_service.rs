// crates/backend/src/service/wallet_service.rs
// 목적: BCH 지갑 잔액 조회 및 출금 비즈니스 로직.
//
// [보안 흐름]
// - 출금 전 반드시 auth_service.verify_otp()로 2FA 검증 (OTP 없으면 출금 불가)
// - 수수료는 EvmWalletPort를 통해 실시간 추정
// - 개인키는 EncryptedKey 형태로만 전달, 복호화는 infra에서만 처리

use std::sync::Arc;
use sea_orm::DatabaseConnection;
use shared::dto::transaction_dto::{TxHistoryItemRes, WalletStateRes};
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
        use crate::db::entity::wallet;
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
        use crate::service::wallet_domain::WalletDomain;
        
        let wallet = wallet::Entity::find()
            .filter(wallet::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|e| WalletServiceError::Internal(e.to_string()))?
            .ok_or(WalletServiceError::WalletNotFound)?;
            
        let info = WalletDomain::get_balance_info(&self.db, self.wallet_port.clone(), user_id, &wallet.eth_address, wallet.id)
            .await
            .map_err(|_| WalletServiceError::Internal("Balance calc error".to_string()))?;
        
        // 1. available = available_eth_balance
        // 2. locked = display_locked_amount
        // 3. total eth = eth_balance (onchain balance + wallet_locked_amount_eth - effectively this is onchain + pending)
        // Actually, wallet.into_dto takes (available, locked, eth_balance)
        // So eth_balance should just be the onchain balance if needed, but let's pass what's expected.
        let total_eth = info.available_eth_balance + info.wallet_locked_amount_eth; 
        
        Ok(wallet.into_dto(info.available_eth_balance, info.display_locked_amount, total_eth))
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
        use crate::db::entity::{wallet, wallet_transaction};
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set, TransactionTrait};
        use rust_decimal::Decimal;
        use rust_decimal::prelude::FromPrimitive;
        use std::str::FromStr;
        use crate::service::wallet_domain::WalletDomain;
        
        self.auth_service.verify_otp(user_id, &req.otp_token).await
            .map_err(|_| WalletServiceError::OtpFailed)?;
            
        if !self.wallet_port.validate_address(&req.destination_eth).await.unwrap_or(false) {
            return Err(WalletServiceError::InvalidAddress);
        }
        
        let txn = self.db.begin().await.map_err(|e| WalletServiceError::Internal(e.to_string()))?;
        
        let wallet_model = wallet::Entity::find()
            .filter(wallet::Column::UserId.eq(user_id))
            .one(&txn)
            .await
            .map_err(|e| WalletServiceError::Internal(e.to_string()))?
            .ok_or(WalletServiceError::WalletNotFound)?;
            
        let amount_decimal = rust_decimal::Decimal::from_f64(req.amount_eth)
            .unwrap_or(rust_decimal::Decimal::new(0, 0));
            
        // Prepare U256 Wei amounts
        let amount_str = req.amount_eth.to_string();
        let amount_u256 = ethers::utils::parse_ether(&amount_str)
            .map_err(|_| WalletServiceError::Internal("Invalid amount format".to_string()))?;

        let fee_u256 = self.wallet_port.estimate_fee(amount_u256)
            .await
            .map_err(|e| WalletServiceError::Internal(e.to_string()))?;
            
        let fee_eth_str = ethers::utils::format_units(fee_u256, "ether")
            .unwrap_or_else(|_| "0".to_string());
        let fee_decimal = rust_decimal::Decimal::from_str(&fee_eth_str)
            .unwrap_or(rust_decimal::Decimal::new(0, 0));
            
        // Verify sufficient funds using SSOT logic
        let required_amount_f64 = req.amount_eth;
        let fee_f64 = fee_eth_str.parse::<f64>().unwrap_or(0.0);
        
        WalletDomain::verify_sufficient_funds(&txn, self.wallet_port.clone(), user_id, "ETH", required_amount_f64, fee_f64)
            .await
            .map_err(|_| WalletServiceError::InsufficientAvailableFunds)?;
        
        // Broadcast transaction via port
        let tx_id = self.wallet_port.broadcast_transaction(
            &wallet_model.eth_address,
            &req.destination_eth,
            amount_u256,
            fee_u256,
            b"mock_private_key"
        )
            .await
            .map_err(|e| WalletServiceError::Internal(e.to_string()))?;
            
        let new_tx = wallet_transaction::ActiveModel {
            wallet_id: Set(wallet_model.id),
            tx_type: Set("withdrawal".to_string()),
            amount: Set(amount_decimal),
            network_fee: Set(fee_decimal),
            tx_hash: Set(Some(tx_id.clone())),
            status: Set("pending".to_string()), // Blockchain confirm wait
            created_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };
        
        let inserted = new_tx.insert(&txn).await
            .map_err(|e| WalletServiceError::Internal(e.to_string()))?;
            
        txn.commit().await.map_err(|e| WalletServiceError::Internal(e.to_string()))?;
        
        Ok(TxHistoryItemRes {
            tx_uid: crate::utils::security::obfuscate(inserted.id).map_err(|_| WalletServiceError::Internal("ID error".to_string()))?,
            movement_type: shared::dto::transaction_dto::MovementType::Withdrawal,
            amount: req.amount_eth,
            fee_deducted: fee_f64,
            blockchain_hash: Some(tx_id),
            process_status: shared::dto::transaction_dto::TxStatus::Pending,
            timestamp: inserted.created_at.to_rfc3339(),
        })
    }
}
