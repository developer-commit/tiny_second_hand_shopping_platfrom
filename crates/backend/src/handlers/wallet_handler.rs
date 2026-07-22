// crates/backend/src/handlers/wallet_handler.rs
// 목적: 지갑 조회 및 출금 HTTP 핸들러.
// 출금 핸들러는 otp_token 필드를 포함한 WithdrawReq를 그대로 Service에 전달하며
// Service → AuthService.verify_otp()에서 2FA를 검증합니다.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use shared::dto::transaction_dto::{TxHistoryItemRes, WalletStateRes, WithdrawReq};
use crate::state::AppState;
use crate::utils::auth::Claims;

/// GET /v1/wallet (인증 필요)
pub async fn get_wallet(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<WalletStateRes>, StatusCode> {
    todo!("claims → user_id → state.wallet_service.get_wallet_state(user_id).await → Json")
}

/// POST /v1/wallet/withdraw (인증 필요 + 2FA 내부 검증)
pub async fn withdraw(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<WithdrawReq>,
) -> Result<Json<TxHistoryItemRes>, StatusCode> {
    todo!("validate(req) → claims → user_id → state.wallet_service.withdraw(user_id, req).await → Json")
}

/// GET /v1/wallet/history (인증 필요)
pub async fn get_tx_history(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<TxHistoryItemRes>>, StatusCode> {
    todo!("claims → user_id → state.wallet_service.get_tx_history(user_id).await → Json")
}
