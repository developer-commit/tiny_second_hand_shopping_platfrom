// crates/backend/src/handlers/wallet_handler.rs
// 목적: 지갑 조회 및 출금 HTTP 핸들러.
// 출금 핸들러는 otp_token 필드를 포함한 WithdrawReq를 그대로 Service에 전달하며
// Service → AuthService.verify_otp()에서 2FA를 검증합니다.

use crate::state::AppState;
use crate::utils::auth::Claims;
use crate::utils::error::AppError;
use axum::{extract::State, http::StatusCode, response::Json};
use shared::dto::transaction_dto::{EthWithdrawReq, TxHistoryItemRes, WalletStateRes};

use crate::utils::security::deobfuscate;

/// GET /v1/wallet (인증 필요)
pub async fn get_wallet(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<WalletStateRes>, AppError> {
    let user_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    state
        .wallet_service
        .get_wallet_state(user_id)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("get_wallet error: {:?}", e);
            AppError::Internal
        })
}

/// GET /v1/wallet/history (인증 필요)
pub async fn get_tx_history(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<TxHistoryItemRes>>, AppError> {
    let user_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    state
        .wallet_service
        .get_tx_history(user_id)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("get_tx_history error: {:?}", e);
            AppError::Internal
        })
}

/// POST /v1/wallet/eth/withdraw (인증 필요 + 2FA 내부 검증)
pub async fn eth_withdraw(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<EthWithdrawReq>,
) -> Result<Json<TxHistoryItemRes>, AppError> {
    use validator::Validate;
    req.validate()
        .map_err(|_| AppError::BadRequest("Invalid request".to_string()))?;

    let user_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    state
        .wallet_service
        .eth_withdraw(user_id, req)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("eth_withdraw error: {:?}", e);
            AppError::Internal
        })
}
