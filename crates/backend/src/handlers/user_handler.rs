// crates/backend/src/handlers/user_handler.rs
// 목적: 사용자 프로필 HTTP 핸들러.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use shared::dto::user_dto::{UpdateProfileReq, UserProfileRes};
use crate::state::AppState;
use crate::utils::auth::Claims;

/// GET /v1/users/me
pub async fn get_my_profile(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<UserProfileRes>, StatusCode> {
    todo!("claims.sub → deobfuscate → user_id → state.user_service.get_profile(user_id).await → Json")
}

/// PATCH /v1/users/me
pub async fn update_my_profile(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<UpdateProfileReq>,
) -> Result<Json<UserProfileRes>, StatusCode> {
    todo!("claims → user_id → state.user_service.update_profile(user_id, req).await → Json")
}
