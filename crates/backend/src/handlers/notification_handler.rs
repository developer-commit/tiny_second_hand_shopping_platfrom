// crates/backend/src/handlers/notification_handler.rs
use crate::state::AppState;
use crate::utils::auth::Claims;
use crate::utils::error::AppError;
use crate::utils::security::deobfuscate;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use shared::dto::noti_dto::NotificationRes;

/// GET /v1/notifications (인증 필요)
pub async fn get_notifications(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<NotificationRes>>, AppError> {
    let user_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    state
        .notification_service
        .get_notifications(user_id)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("get_notifications error: {:?}", e);
            AppError::Internal
        })
}

/// PATCH /v1/notifications/:noti_uid/read (인증 필요)
pub async fn mark_notification_read(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(noti_uid): Path<String>,
) -> Result<StatusCode, AppError> {
    let user_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let noti_id =
        deobfuscate(&noti_uid).map_err(|_| AppError::NotFound("Not found".to_string()))?;

    state
        .notification_service
        .mark_as_read(user_id, noti_id)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|e| {
            tracing::error!("mark_notification_read error: {:?}", e);
            AppError::Internal
        })
}
