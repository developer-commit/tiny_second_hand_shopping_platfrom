// crates/backend/src/handlers/admin_handler.rs
// 목적: 관리자 전용 핸들러.
// [보안] 이 모듈의 모든 핸들러는 router.rs에서 rbac::require_admin 레이어로 감쌉니다.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use shared::dto::admin_dto::{ForceSettleReq, PlatformStatsRes};
use crate::state::AppState;
use crate::utils::auth::Claims;

/// GET /v1/admin/stats (관리자 전용)
pub async fn get_platform_stats(
    State(state): State<AppState>,
) -> Result<Json<PlatformStatsRes>, StatusCode> {
    todo!("state.admin_service.get_platform_stats().await → Json")
}

/// POST /v1/admin/escrow/:trade_uid/force-settle (관리자 전용)
pub async fn force_settle(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<ForceSettleReq>,
) -> Result<StatusCode, StatusCode> {
    todo!("claims → admin_id → state.admin_service.force_settle(admin_id, req).await → 200")
}
