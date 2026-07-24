// crates/backend/src/handlers/escrow_handler.rs
// 목적: 에스크로 예치/수령/분쟁 HTTP 핸들러.

use crate::utils::error::AppError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use shared::dto::escrow_dto::{DisputeEscrowReq, InitiateEscrowReq, SafeTradeStatusRes};
use crate::state::AppState;
use crate::utils::auth::Claims;

use crate::utils::security::deobfuscate;

/// POST /v1/escrow (인증 필요)
pub async fn initiate_escrow(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<InitiateEscrowReq>,
) -> Result<(StatusCode, Json<SafeTradeStatusRes>), AppError> {
    let buyer_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    state.escrow_service.initiate(buyer_id, req).await
        .map(|res| (StatusCode::CREATED, Json(res)))
        .map_err(|e| {
            tracing::error!("initiate_escrow error: {:?}", e);
            map_escrow_error(e)
        })
}

/// POST /v1/escrow/:trade_uid/confirm (인증 필요)
pub async fn confirm_escrow(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(trade_uid): Path<String>,
) -> Result<StatusCode, AppError> {
    let buyer_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let trade_id = deobfuscate(&trade_uid).map_err(|_| AppError::NotFound("Not found".to_string()))?;
    
    state.escrow_service.confirm(buyer_id, trade_id).await
        .map(|_| StatusCode::OK)
        .map_err(|e| {
            tracing::error!("confirm_escrow error: {:?}", e);
            map_escrow_error(e)
        })
}

/// POST /v1/escrow/:trade_uid/deposit (인증 필요)
pub async fn deposit_escrow(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(trade_uid): Path<String>,
) -> Result<StatusCode, AppError> {
    let buyer_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let trade_id = deobfuscate(&trade_uid).map_err(|_| AppError::NotFound("Not found".to_string()))?;
    
    state.escrow_service.deposit(buyer_id, trade_id).await
        .map(|_| StatusCode::OK)
        .map_err(|e| {
            tracing::error!("deposit_escrow error: {:?}", e);
            map_escrow_error(e)
        })
}

/// POST /v1/escrow/:trade_uid/dispute (인증 필요)
pub async fn dispute_escrow(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(trade_uid): Path<String>,
    Json(req): Json<DisputeEscrowReq>,
) -> Result<StatusCode, AppError> {
    let buyer_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let trade_id = deobfuscate(&trade_uid).map_err(|_| AppError::NotFound("Not found".to_string()))?;
    
    state.escrow_service.dispute(buyer_id, trade_id, req).await
        .map(|_| StatusCode::OK)
        .map_err(|e| {
            tracing::error!("dispute_escrow error: {:?}", e);
            map_escrow_error(e)
        })
}

/// GET /v1/escrow/:trade_uid (인증 필요)
pub async fn get_escrow(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(trade_uid): Path<String>,
) -> Result<Json<SafeTradeStatusRes>, AppError> {
    let user_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let trade_id = deobfuscate(&trade_uid).map_err(|_| AppError::NotFound("Not found".to_string()))?;
    
    state.escrow_service.get_trade(user_id, trade_id).await
        .map(Json)
        .map_err(|e| {
            tracing::error!("get_escrow error: {:?}", e);
            map_escrow_error(e)
        })
}

fn map_escrow_error(e: crate::service::traits::EscrowServiceError) -> AppError {
    match e {
        crate::service::traits::EscrowServiceError::ProductNotFound => AppError::NotFound("Product not found".to_string()),
        crate::service::traits::EscrowServiceError::EscrowNotInitiated => AppError::NotFound("에스크로가 아직 시작되지 않았습니다.".to_string()),
        crate::service::traits::EscrowServiceError::TradeNotFound => AppError::NotFound("Trade not found".to_string()),
        crate::service::traits::EscrowServiceError::InsufficientBalance => AppError::BadRequest("Insufficient balance".to_string()),
        crate::service::traits::EscrowServiceError::DuplicateTrade => AppError::BadRequest("Duplicate trade".to_string()),
        crate::service::traits::EscrowServiceError::InvalidState(s) => AppError::BadRequest(format!("Invalid state: {}", s)),
        crate::service::traits::EscrowServiceError::Forbidden => AppError::Forbidden,
        crate::service::traits::EscrowServiceError::InvalidWalletAddress => AppError::BadRequest("Invalid wallet address".to_string()),
        crate::service::traits::EscrowServiceError::ContractRevert(msg) => AppError::BadRequest(format!("Contract revert: {}", msg)),
        crate::service::traits::EscrowServiceError::RpcError(msg) => AppError::BadGateway(format!("RPC error: {}", msg)),
        crate::service::traits::EscrowServiceError::Internal(_) => AppError::Internal,
    }
}
