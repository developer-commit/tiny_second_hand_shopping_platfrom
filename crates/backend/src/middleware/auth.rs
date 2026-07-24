// crates/backend/src/middleware/auth.rs
// 목적: JWT 인증 미들웨어.
// Authorization: Bearer <token> 헤더에서 JWT를 추출하고 Claims를 Extension에 주입합니다.
// Claims가 주입된 이후 핸들러에서 axum::Extension<Claims>로 추출하여 사용합니다.
//
// [보안 흐름]
// 1. Authorization 헤더 추출
// 2. verify_token()으로 서명 및 만료 검증
// 3. Claims.status 확인 — dormant/suspended 계정 차단
// 4. Extension에 Claims 주입 → 핸들러로 전달

use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::utils::auth::{verify_token, Claims};
use crate::state::AppState;

/// JWT 인증 미들웨어 함수
/// `router.layer(axum::middleware::from_fn_with_state(state, authenticate))`로 적용
pub async fn authenticate(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|val| val.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));

    let query_token = request
        .uri()
        .query()
        .unwrap_or("")
        .split('&')
        .find(|q| q.starts_with("token="))
        .map(|q| q.trim_start_matches("token="));

    let token = match auth_header.or(query_token) {
        Some(t) => t,
        None => {
            tracing::warn!("Missing or invalid Authorization header or token query parameter");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let claims = match verify_token(token, &state.jwt_secret) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Token verification failed: {:?}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    if claims.status == "dormant" || claims.status == "suspended" {
        tracing::warn!("Access denied for user status: {}", claims.status);
        return Err(StatusCode::FORBIDDEN);
    }

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}
