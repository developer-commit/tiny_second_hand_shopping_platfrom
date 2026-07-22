// crates/backend/src/main.rs
// 목적: 백엔드 서버 진입점.
// 의존성 트리 조립: DB 연결 → Redis 연결 → Adapter → Service → AppState → Router → Server

mod db;
mod utils;
mod ports;
mod service;
mod infra;
mod handlers;
mod middleware;
mod state;
mod router;

use dotenvy::dotenv;
use sea_orm::Database;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::router::build_router;

#[tokio::main]
async fn main() {
    // ─── 환경 변수 로드 ───────────────────────────────────────────────────────
    dotenv().ok();

    // ─── 로깅 초기화 ──────────────────────────────────────────────────────────
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("서버 시작 중...");

    // ─── 의존성 트리 조립 ─────────────────────────────────────────────────────
    todo!(
        "1. DATABASE_URL → sea_orm::Database::connect() → DatabaseConnection\n\
        2. REDIS_URL → redis::Client → ConnectionManager → RedisPubSubAdapter\n\
        3. BCH_API_URL, AES_KEY → BchWalletAdapter, BchNetworkAdapter\n\
        4. JWT_SECRET → JwtSecret(SecretString::new(...))\n\
        5. 각 Service::new(db, adapters) 조립\n\
        6. AppState::new(...)\n\
        7. build_router(state)\n\
        8. tokio-cron-scheduler: EscrowService.auto_confirm_expired_trades 스케줄 등록\n\
        9. TcpListener::bind('0.0.0.0:8080') → axum::serve(listener, app).await"
    )
}
