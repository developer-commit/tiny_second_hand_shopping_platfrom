// crates/backend/src/handlers/admin_handler.rs
// 목적: 관리자 전용 핸들러.
// [보안] 이 모듈의 모든 핸들러는 router.rs에서 rbac::require_admin 레이어로 감쌉니다.

use crate::state::AppState;
use crate::utils::auth::Claims;
use crate::utils::error::AppError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use shared::dto::admin_dto::{
    AdminReportListRes, BanUserReq, ForceSettleReq, HideProductReq, PlatformStatsRes,
};

/// GET /v1/admin/stats (관리자 전용)
pub async fn get_platform_stats(
    State(state): State<AppState>,
) -> Result<Json<PlatformStatsRes>, AppError> {
    match state.admin_service.get_platform_stats().await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(AppError::Internal),
    }
}

/// POST /v1/admin/escrow/force-settle (관리자 전용)
pub async fn force_settle(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<ForceSettleReq>,
) -> Result<StatusCode, AppError> {
    // claims.sub contains the deobfuscated admin_id if we assume auth middleware already processed it
    let admin_id = crate::utils::security::deobfuscate(&claims.sub).unwrap_or(0); // Alternatively parse directly if sub is just stringified id

    match state.admin_service.force_settle(admin_id, req).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(crate::service::admin_service::AdminServiceError::TradeNotFound) => {
            Err(AppError::NotFound("Not found".to_string()))
        }
        Err(crate::service::admin_service::AdminServiceError::AlreadySettled) => {
            Err(AppError::BadRequest("Already settled".to_string()))
        }
        Err(_) => Err(AppError::Internal),
    }
}

/// POST /v1/admin/users/:user_uid/ban (관리자 전용)
pub async fn ban_user(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(user_uid): Path<String>,
    Json(req): Json<BanUserReq>,
) -> Result<StatusCode, AppError> {
    let admin_id = crate::utils::security::deobfuscate(&claims.sub).unwrap_or(0);
    let user_id = crate::utils::security::deobfuscate(&user_uid)
        .map_err(|_| AppError::NotFound("User not found".to_string()))?;

    match state.admin_service.ban_user(admin_id, user_id, req).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(AppError::Internal),
    }
}

/// POST /v1/admin/products/:item_uid/hide (관리자 전용)
pub async fn hide_product(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(item_uid): Path<String>,
    Json(req): Json<HideProductReq>,
) -> Result<StatusCode, AppError> {
    let admin_id = crate::utils::security::deobfuscate(&claims.sub).unwrap_or(0);
    let product_id = crate::utils::security::deobfuscate(&item_uid)
        .map_err(|_| AppError::NotFound("Product not found".to_string()))?;

    match state
        .admin_service
        .hide_product(admin_id, product_id, req)
        .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(AppError::Internal),
    }
}

/// GET /v1/admin/reports (관리자 전용)
pub async fn list_reports(
    State(state): State<AppState>,
) -> Result<Json<AdminReportListRes>, AppError> {
    match state.admin_service.list_reports().await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(AppError::Internal),
    }
}
