// crates/backend/src/handlers/report_handler.rs
use crate::utils::error::AppError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use shared::dto::report_dto::{ReportAckRes, SubmitReportReq};
use crate::state::AppState;
use crate::utils::auth::Claims;
use crate::utils::security::deobfuscate;

/// POST /v1/products/:item_uid/reports (인증 필요)
pub async fn submit_report(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(item_uid): Path<String>,
    Json(req): Json<SubmitReportReq>,
) -> Result<(StatusCode, Json<ReportAckRes>), AppError> {
    use validator::Validate;
    req.validate().map_err(|_| AppError::BadRequest("Invalid request".to_string()))?;

    let reporter_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let product_id = deobfuscate(&item_uid).map_err(|_| AppError::NotFound("Not found".to_string()))?;

    state.report_service.submit_report(reporter_id, product_id, req).await
        .map(|res| (StatusCode::CREATED, Json(res)))
        .map_err(|e| {
            tracing::error!("submit_report error: {:?}", e);
            match e {
                crate::service::report_service::ReportServiceError::SelfReport => AppError::Forbidden,
                _ => AppError::Internal,
            }
        })
}
