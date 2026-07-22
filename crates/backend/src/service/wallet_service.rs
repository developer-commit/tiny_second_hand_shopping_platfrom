// crates/backend/src/service/wallet_service.rs
// 목적: BCH 지갑 잔액 조회 및 출금 비즈니스 로직.
//
// [보안 흐름]
// - 출금 전 반드시 auth_service.verify_otp()로 2FA 검증 (OTP 없으면 출금 불가)
// - 수수료는 BchWalletPort를 통해 실시간 추정
// - 개인키는 EncryptedKey 형태로만 전달, 복호화는 infra에서만 처리

use std::sync::Arc;
use sea_orm::DatabaseConnection;
use shared::dto::transaction_dto::{TxHistoryItemRes, WalletStateRes, WithdrawReq};
use thiserror::Error;
use crate::ports::wallet_port::BchWalletPort;
use crate::service::auth_service::AuthService;

#[derive(Debug, Error)]
pub enum WalletServiceError {
    #[error("지갑을 찾을 수 없습니다.")]
    WalletNotFound,
    #[error("잔액이 부족합니다.")]
    InsufficientBalance,
    #[error("2FA 인증 실패")]
    OtpFailed,
    #[error("출금 주소가 유효하지 않습니다.")]
    InvalidAddress,
    #[error("내부 서버 오류: {0}")]
    Internal(String),
}

pub struct WalletService {
    db: DatabaseConnection,
    wallet_port: Arc<dyn BchWalletPort>,
    auth_service: Arc<AuthService>,
}

impl WalletService {
    pub fn new(
        db: DatabaseConnection,
        wallet_port: Arc<dyn BchWalletPort>,
        auth_service: Arc<AuthService>,
    ) -> Self {
        WalletService { db, wallet_port, auth_service }
    }

    /// 지갑 상태 조회 (GET /wallet)
    pub async fn get_wallet_state(
        &self,
        user_id: i64,
    ) -> Result<WalletStateRes, WalletServiceError> {
        todo!("wallets SELECT + escrow_trades SUM(amount) WHERE status='deposited' → into_dto")
    }

    /// 출금 처리 (POST /wallet/withdraw)
    /// 순서: OTP 검증 → 주소 검증 → 잔액 확인 → 수수료 추정 → TX 브로드캐스트 → 내역 저장
    pub async fn withdraw(
        &self,
        user_id: i64,
        req: WithdrawReq,
    ) -> Result<TxHistoryItemRes, WalletServiceError> {
        todo!("auth_service.verify_otp → wallet_port.validate_address → 잔액 확인 → wallet_port.estimate_fee → wallet_port.broadcast_transaction → wallet_transactions INSERT")
    }

    /// 트랜잭션 내역 조회
    pub async fn get_tx_history(
        &self,
        user_id: i64,
    ) -> Result<Vec<TxHistoryItemRes>, WalletServiceError> {
        todo!("wallet_transactions SELECT WHERE wallet.user_id = user_id → TryFrom<Model>")
    }
}
