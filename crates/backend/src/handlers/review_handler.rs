// crates/backend/src/handlers/review_handler.rs
use crate::state::AppState;
use crate::utils::auth::Claims;
use crate::utils::error::AppError;
use crate::utils::security::deobfuscate;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use shared::dto::review_dto::{ReviewRes, SubmitReviewReq};

/// POST /v1/escrow/:trade_uid/reviews (인증 필요)
pub async fn submit_review(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(trade_uid): Path<String>,
    Json(req): Json<SubmitReviewReq>,
) -> Result<(StatusCode, Json<ReviewRes>), AppError> {
    use validator::Validate;
    req.validate()
        .map_err(|_| AppError::BadRequest("Invalid request".to_string()))?;

    let reviewer_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let trade_id =
        deobfuscate(&trade_uid).map_err(|_| AppError::NotFound("Not found".to_string()))?;

    state
        .review_service
        .submit_review(reviewer_id, trade_id, req)
        .await
        .map(|res| (StatusCode::CREATED, Json(res)))
        .map_err(|e| {
            tracing::error!("submit_review error: {:?}", e);
            AppError::Internal
        })
}
