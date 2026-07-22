// crates/backend/src/handlers/escrow_handler.rs
// 목적: 에스크로 예치/수령/분쟁 HTTP 핸들러.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use shared::dto::escrow_dto::{DisputeEscrowReq, InitiateEscrowReq, SafeTradeStatusRes};
use crate::state::AppState;
use crate::utils::auth::Claims;

/// POST /v1/escrow (인증 필요)
pub async fn initiate_escrow(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<InitiateEscrowReq>,
) -> Result<(StatusCode, Json<SafeTradeStatusRes>), StatusCode> {
    todo!("claims → buyer_id → state.escrow_service.initiate(buyer_id, req).await → 201")
}

/// POST /v1/escrow/:trade_uid/confirm (인증 필요)
pub async fn confirm_escrow(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(trade_uid): Path<String>,
) -> Result<StatusCode, StatusCode> {
    todo!("claims → buyer_id, deobfuscate(trade_uid) → state.escrow_service.confirm(buyer_id, trade_id).await")
}

/// POST /v1/escrow/:trade_uid/dispute (인증 필요)
pub async fn dispute_escrow(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(trade_uid): Path<String>,
    Json(req): Json<DisputeEscrowReq>,
) -> Result<StatusCode, StatusCode> {
    todo!("claims → buyer_id, deobfuscate(trade_uid) → state.escrow_service.dispute(buyer_id, trade_id, req).await")
}
