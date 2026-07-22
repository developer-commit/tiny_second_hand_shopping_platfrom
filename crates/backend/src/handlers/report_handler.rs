// crates/backend/src/handlers/report_handler.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use shared::dto::report_dto::{ReportAckRes, SubmitReportReq};
use crate::state::AppState;
use crate::utils::auth::Claims;

/// POST /v1/products/:item_uid/reports (인증 필요)
pub async fn submit_report(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(item_uid): Path<String>,
    Json(req): Json<SubmitReportReq>,
) -> Result<(StatusCode, Json<ReportAckRes>), StatusCode> {
    todo!("validate(req) → claims → reporter_id, deobfuscate(item_uid) → state.report_service.submit_report(reporter_id, product_id, req).await → 201")
}
