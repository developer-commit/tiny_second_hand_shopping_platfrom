// crates/backend/src/handlers/user_handler.rs
// 목적: 사용자 프로필 HTTP 핸들러.

use crate::utils::error::AppError;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use shared::dto::user_dto::{UpdateProfileReq, UserProfileRes};
use crate::state::AppState;
use crate::utils::auth::Claims;

use crate::utils::security::deobfuscate;

/// GET /v1/users/me
pub async fn get_my_profile(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<UserProfileRes>, AppError> {
    let user_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    state.user_service.get_profile(user_id).await
        .map(Json)
        .map_err(|e| {
            tracing::error!("get_profile error: {:?}", e);
            AppError::Internal
        })
}

/// PATCH /v1/users/me
pub async fn update_my_profile(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<UpdateProfileReq>,
) -> Result<Json<UserProfileRes>, AppError> {
    let user_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    state.user_service.update_profile(user_id, req).await
        .map(Json)
        .map_err(|e| {
            tracing::error!("update_profile error: {:?}", e);
            AppError::Internal
        })
}
