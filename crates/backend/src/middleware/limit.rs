use crate::state::AppState;
use crate::utils::auth::{Claims, verify_token};
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::Response,
};

use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const ONE_HOUR: Duration = Duration::from_secs(3600);

// ================= =================
// 2. 미들웨어
// ================= =================
pub async fn email_rate_limit_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let client_ip = extract_ip(&headers);
    let now = Instant::now();

    // 1. 짧은 블록 스코프 안에서 Lock 잡고 카운트 검사
    {
        let mut limits_map = state.email_rate_limits.lock().unwrap();
        let entry = limits_map.entry(client_ip).or_insert((0, now));

        // 1시간이 지났으면 리셋
        if now.duration_since(entry.1) > ONE_HOUR {
            entry.0 = 0;
            entry.1 = now;
        }

        // 5회 초과 시 핸들러를 호출하지 않고 즉시 429 반환 (Early Return)
        if entry.0 >= 5 {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        entry.0 += 1;
    } // <-- 여기서 MutexGuard가 Drop되면서 락이 완전히 해제됨!

    // 2. 통과했으므로 비동기로 다음 핸들러 실행
    Ok(next.run(request).await)
}

// IP 추출 헬퍼 함수
fn extract_ip(headers: &HeaderMap) -> IpAddr {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|ip| ip.trim().parse().ok())
        .unwrap_or_else(|| [127, 0, 0, 1].into())
}
