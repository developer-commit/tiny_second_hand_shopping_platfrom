// crates/backend/src/handlers/auth_handler.rs
// 목적: 인증 관련 HTTP 핸들러 (Controller Layer).
// HTTP I/O만 담당하며 비즈니스 로직은 AuthService에 위임합니다.
// 이 계층에서는 JSON 역직렬화, validator 호출, HTTP 상태코드 매핑만 수행합니다.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use validator::Validate;
use shared::dto::user_dto::{
    AuthTokenRes, Enable2FaReq, LoginReq, SignUpReq, TwoFaSetupRes,
};
use crate::state::AppState;

/// POST /v1/auth/signup
pub async fn signup(
    State(state): State<AppState>,
    Json(req): Json<SignUpReq>,
) -> Result<StatusCode, StatusCode> {
    req.validate().map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    todo!("state.auth_service.sign_up(req).await → 201 Created 또는 에러 응답 매핑")
}

/// POST /v1/auth/login
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginReq>,
) -> Result<Json<AuthTokenRes>, StatusCode> {
    req.validate().map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    todo!("state.auth_service.login(req).await → Json(token_res) 또는 401 Unauthorized")
}

/// POST /v1/users/me/2fa/setup (인증 필요)
pub async fn setup_2fa(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<crate::utils::auth::Claims>,
) -> Result<Json<TwoFaSetupRes>, StatusCode> {
    todo!("claims.sub → deobfuscate → user_id → state.auth_service.setup_2fa(user_id).await")
}

/// POST /v1/users/me/2fa/enable (인증 필요)
pub async fn enable_2fa(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<crate::utils::auth::Claims>,
    Json(req): Json<Enable2FaReq>,
) -> Result<StatusCode, StatusCode> {
    todo!("claims → user_id → state.auth_service.enable_2fa(user_id, req).await")
}
