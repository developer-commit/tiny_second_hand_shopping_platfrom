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

    // ─── 1. DATABASE_URL ──────────────────────────────────────────────────────
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&db_url).await.expect("Failed to connect to DB");

    // ─── 2. REDIS_URL ─────────────────────────────────────────────────────────
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let redis_client = redis::Client::open(redis_url).expect("Invalid Redis URL");
    let redis_conn = redis::aio::ConnectionManager::new(redis_client.clone()).await.expect("Failed to create Redis connection manager");
    let pubsub_adapter = std::sync::Arc::new(crate::infra::redis_pubsub_adapter::RedisPubSubAdapter::new(redis_conn));

    // ─── 3. ETH_RPC_URL, MASTER_WALLET_KEY & AES_KEY ──────────────────────────
    let eth_rpc_url = std::env::var("ETH_RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:8545".to_string());
    let master_wallet_key = std::env::var("MASTER_WALLET_KEY").unwrap_or_else(|_| "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string());
    // For demo purposes, we will use a dummy 32-byte key if not set.
    let aes_key_str = std::env::var("AES_KEY").unwrap_or_else(|_| "01234567890123456789012345678901".to_string());
    let mut aes_key = [0u8; 32];
    let key_bytes = aes_key_str.as_bytes();
    for i in 0..std::cmp::min(32, key_bytes.len()) {
        aes_key[i] = key_bytes[i];
    }
    
    let wallet_adapter = std::sync::Arc::new(crate::infra::evm_wallet_adapter::EvmWalletAdapter::new(aes_key, eth_rpc_url.clone()));
    
    let blockchain_manager = crate::utils::blockchain::BlockchainManager::new(&eth_rpc_url, &master_wallet_key).await.expect("Failed to initialize BlockchainManager");
    let blockchain_manager_arc = std::sync::Arc::new(blockchain_manager);
    
    //let verification_adapter = std::sync::Arc::new(crate::infra::verification_adapter::VerificationAdapter::new());
    let verification_adapter = std::sync::Arc::new(crate::infra::mock_verification_adapter::MockVerificationAdapter::new());

    // ─── 4. JWT_SECRET ────────────────────────────────────────────────────────
    let jwt_secret_str = std::env::var("JWT_SECRET").unwrap_or_else(|_| "super-secret-jwt-key-for-dev-only-change-in-prod".to_string());
    let jwt_secret = crate::utils::auth::JwtSecret(crate::utils::security::SecretString::new(jwt_secret_str));
    let jwt_secret_arc = std::sync::Arc::new(jwt_secret);
    use crate::state::DbClone;

    let auth_service_arc: std::sync::Arc<dyn crate::service::traits::AuthServiceTrait> = std::sync::Arc::new(crate::service::auth_service::AuthService::new(
        db.clone_conn(),
        jwt_secret_arc.clone(),
        verification_adapter.clone(),
        wallet_adapter.clone(),
    ));
    let user_service_arc: std::sync::Arc<dyn crate::service::traits::UserServiceTrait> = std::sync::Arc::new(crate::service::user_service::UserService::new(db.clone_conn()));
    
    let product_service_arc: std::sync::Arc<dyn crate::service::traits::ProductServiceTrait> = std::sync::Arc::new(crate::service::product_service::ProductService::new(db.clone_conn()));
    let wallet_service_arc: std::sync::Arc<dyn crate::service::traits::WalletServiceTrait> = std::sync::Arc::new(crate::service::wallet_service::WalletService::new(
        db.clone_conn(),
        wallet_adapter.clone(),
        auth_service_arc.clone(),
    ));
    
    let notification_service_impl = std::sync::Arc::new(crate::service::notification_service::NotificationService::new(db.clone_conn(), pubsub_adapter.clone()));
    let notification_service_arc: std::sync::Arc<dyn crate::service::traits::NotificationServiceTrait> = notification_service_impl.clone();
    let notification_port_arc: std::sync::Arc<dyn crate::ports::notification_port::NotificationPort> = notification_service_impl;
    
    let bch_escrow_service: std::sync::Arc<dyn crate::service::traits::EscrowServiceTrait> = std::sync::Arc::new(crate::service::escrow_service::EscrowService::new(
        db.clone_conn(),
        notification_port_arc.clone(),
    ));

    use ethers::core::types::Address;
    let contract_address: Address = std::env::var("ETH_CONTRACT_ADDRESS").unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string()).parse().expect("Invalid ETH_CONTRACT_ADDRESS");
    
    let eth_escrow_service: std::sync::Arc<dyn crate::service::traits::EscrowServiceTrait> = std::sync::Arc::new(crate::service::eth_escrow_service::EthEscrowService::new(
        db.clone_conn(),
        notification_port_arc.clone(),
        blockchain_manager_arc.clone(),
        contract_address,
    ));

    let escrow_service_arc: std::sync::Arc<dyn crate::service::traits::EscrowServiceTrait> = std::sync::Arc::new(crate::service::escrow_dispatcher::EscrowDispatcherService::new(
        db.clone_conn(),
        bch_escrow_service,
        eth_escrow_service,
    ));
    
    let chat_service_arc: std::sync::Arc<dyn crate::service::traits::ChatServiceTrait> = std::sync::Arc::new(crate::service::chat_service::ChatService::new(db.clone_conn(), pubsub_adapter.clone(), notification_service_arc.clone()));
    let review_service_arc: std::sync::Arc<dyn crate::service::traits::ReviewServiceTrait> = std::sync::Arc::new(crate::service::review_service::ReviewService::new(db.clone_conn(), user_service_arc.clone()));
    let report_service_arc: std::sync::Arc<dyn crate::service::traits::ReportServiceTrait> = std::sync::Arc::new(crate::service::report_service::ReportService::new(db.clone_conn()));
    let admin_service_arc: std::sync::Arc<dyn crate::service::traits::AdminServiceTrait> = std::sync::Arc::new(crate::service::admin_service::AdminService::new(db.clone_conn(), wallet_service_arc.clone()));

    // ─── 6. AppState 조립 ─────────────────────────────────────────────────────
    // AppState::new expects JwtSecret, but we have Arc<JwtSecret>. Wait, let's just create a new JwtSecret for AppState.
    let jwt_secret_for_state = crate::utils::auth::JwtSecret(crate::utils::security::SecretString::new(
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "super-secret-jwt-key-for-dev-only-change-in-prod".to_string())
    ));

    let state = crate::state::AppState::new(
        db,
        jwt_secret_for_state,
        auth_service_arc,
        user_service_arc,
        product_service_arc,
        wallet_service_arc,
        escrow_service_arc,
        chat_service_arc,
        review_service_arc,
        report_service_arc,
        admin_service_arc,
        notification_service_arc,
        redis_client.clone(),
        blockchain_manager_arc,
    );

    // ─── 7. Router 조립 ───────────────────────────────────────────────────────
    let app = build_router(state.clone());

    // ─── 8. Cron Scheduler ────────────────────────────────────────────────────
    // EscrowService.auto_confirm_expired_trades 스케줄 등록
    // For now we will spawn a background task loop instead of tokio-cron-scheduler for simplicity without adding deps
    let escrow_service_bg = state.escrow_service.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600)); // Every hour
        loop {
            interval.tick().await;
            tracing::info!("Running auto_confirm_expired_trades...");
            if let Err(e) = escrow_service_bg.auto_confirm_expired_trades().await {
                tracing::error!("auto_confirm_expired_trades error: {:?}", e);
            }
        }
    });

    // ─── 9. TcpListener & Server ──────────────────────────────────────────────
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind TcpListener");
    tracing::info!("Listening on {}", addr);
    
    axum::serve(listener, app).await.expect("Server failed");
}
