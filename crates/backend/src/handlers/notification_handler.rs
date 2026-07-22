// crates/backend/src/handlers/notification_handler.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use shared::dto::noti_dto::NotificationRes;
use crate::state::AppState;
use crate::utils::auth::Claims;

/// GET /v1/notifications (인증 필요)
pub async fn get_notifications(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<NotificationRes>>, StatusCode> {
    todo!("claims → user_id → state.notification_service.get_notifications(user_id).await → Json")
}

/// PATCH /v1/notifications/:noti_uid/read (인증 필요)
pub async fn mark_notification_read(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(noti_uid): Path<String>,
) -> Result<StatusCode, StatusCode> {
    todo!("claims → user_id, deobfuscate(noti_uid) → state.notification_service.mark_as_read(user_id, noti_id).await")
}
