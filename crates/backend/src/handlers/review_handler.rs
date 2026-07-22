// crates/backend/src/handlers/review_handler.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use shared::dto::review_dto::{ReviewRes, SubmitReviewReq};
use crate::state::AppState;
use crate::utils::auth::Claims;

/// POST /v1/escrow/:trade_uid/reviews (인증 필요)
pub async fn submit_review(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(trade_uid): Path<String>,
    Json(req): Json<SubmitReviewReq>,
) -> Result<(StatusCode, Json<ReviewRes>), StatusCode> {
    todo!("validate(req) → claims → reviewer_id, deobfuscate(trade_uid) → state.review_service.submit_review(reviewer_id, trade_id, req).await → 201")
}
