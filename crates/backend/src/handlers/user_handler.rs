// crates/backend/src/handlers/user_handler.rs
// 목적: 사용자 프로필 HTTP 핸들러.

use crate::state::AppState;
use crate::utils::auth::Claims;
use crate::utils::error::AppError;
use axum::{extract::State, http::StatusCode, response::Json};
use shared::dto::user_dto::{PublicUserProfileRes, UpdateProfileReq, UserProfileRes};

use crate::utils::security::deobfuscate;
use axum::extract::Path;

/// GET /v1/users/:user_uid
pub async fn get_public_profile(
    State(state): State<AppState>,
    Path(user_uid): Path<String>,
) -> Result<Json<PublicUserProfileRes>, AppError> {
    let user_id = deobfuscate(&user_uid)
        .map_err(|_| AppError::BadRequest("유효하지 않은 사용자 ID입니다.".into()))?;
    state
        .user_service
        .get_public_profile(user_id)
        .await
        .map(Json)
        .map_err(|e| match e {
            crate::service::user_service::UserServiceError::NotFound => {
                AppError::NotFound("사용자를 찾을 수 없습니다.".into())
            }
            _ => {
                tracing::error!("get_public_profile error: {:?}", e);
                AppError::Internal
            }
        })
}

/// GET /v1/users/me
pub async fn get_my_profile(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<UserProfileRes>, AppError> {
    let user_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    state
        .user_service
        .get_profile(user_id)
        .await
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
    state
        .user_service
        .update_profile(user_id, req)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("update_profile error: {:?}", e);
            AppError::Internal
        })
}
