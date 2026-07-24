// crates/backend/src/state.rs
// 목적: AppState — Axum의 의존성 주입 컨테이너.
// 모든 서비스와 인프라 어댑터를 Arc로 감싸 공유합니다.
// 이 구조체가 Router에 State로 주입되면 모든 핸들러에서 추출할 수 있습니다.
//
// [보안]
// - JwtSecret은 Debug 미구현 SecretString 래퍼로 보관
// - 암호화 키는 직접 노출하지 않고 어댑터 내부에 캡슐화

use crate::{
    service::traits::{
        AdminServiceTrait, AuthServiceTrait, ChatServiceTrait, EscrowServiceTrait,
        NotificationServiceTrait, ProductServiceTrait, ReportServiceTrait, ReviewServiceTrait,
        UserServiceTrait, WalletServiceTrait,
    },
    utils::{auth::JwtSecret, blockchain::BlockchainManager},
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use std::{collections::HashMap, net::IpAddr, sync::Mutex, time::Instant};

/// Axum 전역 공유 상태 — Clone은 Arc 내부 참조만 복제
pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt_secret: Arc<JwtSecret>,

    //ram 자원 사용, 동기 뮤텍스 사용, 조심해서 사용 하기
    pub failed_attempts: Arc<Mutex<HashMap<String, (u32, Instant)>>>, //로그인 요청 횟수
    pub email_rate_limits: Arc<Mutex<HashMap<IpAddr, (u32, Instant)>>>, //이메일 요청횟수

    // ─── Services ────────────────────────────────────────────────────────────
    pub redis_client: redis::Client, // Added for WebSocket PubSub

    pub auth_service: Arc<dyn AuthServiceTrait>,
    pub user_service: Arc<dyn UserServiceTrait>,
    pub product_service: Arc<dyn ProductServiceTrait>,
    pub wallet_service: Arc<dyn WalletServiceTrait>,
    pub escrow_service: Arc<dyn EscrowServiceTrait>,
    pub chat_service: Arc<dyn ChatServiceTrait>,
    pub review_service: Arc<dyn ReviewServiceTrait>,
    pub report_service: Arc<dyn ReportServiceTrait>,
    pub admin_service: Arc<dyn AdminServiceTrait>,
    pub notification_service: Arc<dyn NotificationServiceTrait>,
    pub blockchain_manager: Arc<BlockchainManager>,
}

impl AppState {
    /// AppState 생성 — main.rs에서 의존성 트리를 조립합니다.
    pub fn new(
        db: DatabaseConnection,
        jwt_secret: JwtSecret,
        auth_service: Arc<dyn AuthServiceTrait>,
        user_service: Arc<dyn UserServiceTrait>,
        product_service: Arc<dyn ProductServiceTrait>,
        wallet_service: Arc<dyn WalletServiceTrait>,
        escrow_service: Arc<dyn EscrowServiceTrait>,
        chat_service: Arc<dyn ChatServiceTrait>,
        review_service: Arc<dyn ReviewServiceTrait>,
        report_service: Arc<dyn ReportServiceTrait>,
        admin_service: Arc<dyn AdminServiceTrait>,
        notification_service: Arc<dyn NotificationServiceTrait>,
        redis_client: redis::Client,
        blockchain_manager: Arc<BlockchainManager>,
    ) -> Self {
        AppState {
            db,
            jwt_secret: Arc::new(jwt_secret),
            auth_service,
            user_service,
            product_service,
            wallet_service,
            escrow_service,
            chat_service,
            review_service,
            report_service,
            admin_service,
            notification_service,
            redis_client,
            blockchain_manager,

            failed_attempts: Arc::new(Mutex::new(HashMap::new())),
            email_rate_limits: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

pub trait DbClone {
    fn clone_conn(&self) -> Self;
}

impl DbClone for DatabaseConnection {
    fn clone_conn(&self) -> Self {
        match self {
            DatabaseConnection::SqlxPostgresPoolConnection(pool) => {
                DatabaseConnection::SqlxPostgresPoolConnection(pool.clone())
            }
            #[cfg(test)]
            DatabaseConnection::MockDatabaseConnection(mock) => {
                DatabaseConnection::MockDatabaseConnection(mock.clone())
            }
            DatabaseConnection::Disconnected => DatabaseConnection::Disconnected,
            _ => panic!("Unsupported db variant for clone_conn: {:?}", self),
        }
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone_conn(),
            jwt_secret: self.jwt_secret.clone(),
            auth_service: self.auth_service.clone(),
            user_service: self.user_service.clone(),
            product_service: self.product_service.clone(),
            wallet_service: self.wallet_service.clone(),
            escrow_service: self.escrow_service.clone(),
            chat_service: self.chat_service.clone(),
            review_service: self.review_service.clone(),
            report_service: self.report_service.clone(),
            admin_service: self.admin_service.clone(),
            notification_service: self.notification_service.clone(),
            redis_client: self.redis_client.clone(),
            blockchain_manager: self.blockchain_manager.clone(),

            failed_attempts: self.failed_attempts.clone(),
            email_rate_limits: self.email_rate_limits.clone(),
        }
    }
}
