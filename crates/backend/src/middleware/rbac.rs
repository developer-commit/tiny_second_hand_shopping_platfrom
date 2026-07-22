// crates/backend/src/middleware/rbac.rs
// 목적: 역할 기반 접근 제어(RBAC) 미들웨어.
// admin 전용 라우터 그룹에 레이어로 적용하여
// UserRole::Admin이 아닌 사용자의 접근을 403으로 차단합니다.
//
// [적용 방법]
// Router::new()
//     .route("/admin/...", ...)
//     .layer(axum::middleware::from_fn_with_state(state.clone(), require_admin))

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use crate::utils::auth::{Claims, UserRole};

/// Admin 역할 필수 미들웨어
/// authenticate 미들웨어 이후에 실행됩니다 (Claims Extension이 이미 주입된 상태).
pub async fn require_admin(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if claims.role != UserRole::Admin {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}
