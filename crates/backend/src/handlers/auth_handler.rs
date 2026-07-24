// crates/backend/src/handlers/auth_handler.rs
// 목적: 인증 관련 HTTP 핸들러 (Controller Layer).
// HTTP I/O만 담당하며 비즈니스 로직은 AuthService에 위임합니다.
// 이 계층에서는 JSON 역직렬화, validator 호출, HTTP 상태코드 매핑만 수행합니다.

use crate::utils::error::AppError;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};

// use tower_http::classify::GrpcCode::Ok;
use validator::Validate;
use shared::dto::user_dto::{
    AuthTokenRes, Enable2FaReq, LoginReq, LoginResponse, SendCodeReq, SignUpReq, TwoFaSetupRes,
};
use crate::state::AppState;

use crate::utils::security::deobfuscate;


use std::{
    time::{Duration, Instant},
};

/// POST /v1/auth/signup
pub async fn signup(
    State(state): State<AppState>,
    Json(req): Json<SignUpReq>,
) -> Result<StatusCode, AppError> {
    req.validate().map_err(|_| AppError::BadRequest("Invalid request".to_string()))?;
    state.auth_service.sign_up(req).await
        .map(|_| StatusCode::CREATED)
        .map_err(|e| {
            tracing::error!("Signup error: {:?}", e);
            AppError::Internal
        })
}

/// POST /v1/auth/login
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginReq>,
) -> Result<Json<LoginResponse>, AppError> {
    req.validate().map_err(|_| AppError::BadRequest("Invalid request".to_string()))?;
    const ONE_HOUR: Duration = Duration::from_secs(3600);

    let account_id = req.account_id.clone();
    let now = Instant::now();

    // 1. 사전 차단 여부 검사
    {
        if let Ok(mut res) = state.failed_attempts.lock() {
            let entry = res.entry(account_id.clone()).or_insert((0, now));

            // 1시간이 지났으면 횟수 초기화
            if now.duration_since(entry.1) > ONE_HOUR {
                entry.0 = 0;
                entry.1 = now;
            }

            // 5회 이상 실패 시 차단
            if entry.0 >= 5 {
                return Err(AppError::BadRequest("Too many failed attempts. Try again later.".to_string()));
            }
        }
    }
    // 2. 실제 로그인 시도
    let login_result = state.auth_service.login(req).await.map(Json);

    // 3. 결과 처리
    match login_result {
        Ok(res) => {
            // 로그인 성공: 실패 기록 완전 삭제
            if let Ok(mut res) = state.failed_attempts.lock() {
                res.remove(&account_id);
            }
            
            Ok(res)
        }
        Err(e) => {
            tracing::error!("Login error: {:?}", e);

            // 로그인 실패: 카운트 1 증가 및 시간 갱신
            if let Ok(mut res) = state.failed_attempts.lock() {
                let entry = res.entry(account_id).or_insert((0, now));
                entry.0 += 1;
                entry.1 = Instant::now(); // 실패한 시점 기준으로 갱신
            }

            Err(AppError::BadRequest("login fail".to_string()))
        }
    }
}

/// POST /v1/users/me/2fa/setup (인증 필요)
pub async fn setup_2fa(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<crate::utils::auth::Claims>,
) -> Result<Json<TwoFaSetupRes>, AppError> {
    let user_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    state.auth_service.setup_2fa(user_id).await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Setup 2FA error: {:?}", e);
            AppError::Internal
        })
}

/// POST /v1/users/me/2fa/enable (인증 필요)
pub async fn enable_2fa(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<crate::utils::auth::Claims>,
    Json(req): Json<Enable2FaReq>,
) -> Result<StatusCode, AppError> {
    let user_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    state.auth_service.enable_2fa(user_id, req).await
        .map(|_| StatusCode::OK)
        .map_err(|e| {
            tracing::error!("Enable 2FA error: {:?}", e);
            AppError::Unauthorized
        })
}

/// POST /v1/auth/sendcode
pub async fn sendcode(
    State(state): State<AppState>,
    Json(req): Json<SendCodeReq>,
) -> Result<StatusCode, AppError> {
    req.validate().map_err(|_| AppError::BadRequest("Invalid request".to_string()))?;
    state.auth_service.send_code(req).await
        .map(|_| StatusCode::CREATED)
        .map_err(|e| {
            tracing::error!("send mail error: {:?}", e);
            AppError::Internal
        })
}