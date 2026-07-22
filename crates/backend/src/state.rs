// crates/backend/src/state.rs
// 목적: AppState — Axum의 의존성 주입 컨테이너.
// 모든 서비스와 인프라 어댑터를 Arc로 감싸 공유합니다.
// 이 구조체가 Router에 State로 주입되면 모든 핸들러에서 추출할 수 있습니다.
//
// [보안]
// - JwtSecret은 Debug 미구현 SecretString 래퍼로 보관
// - 암호화 키는 직접 노출하지 않고 어댑터 내부에 캡슐화

use std::sync::Arc;
use sea_orm::DatabaseConnection;
use crate::{
    service::{
        auth_service::AuthService,
        user_service::UserService,
        product_service::ProductService,
        wallet_service::WalletService,
        escrow_service::EscrowService,
        chat_service::ChatService,
        review_service::ReviewService,
        report_service::ReportService,
        admin_service::AdminService,
        notification_service::NotificationService,
    },
    utils::auth::JwtSecret,
};

/// Axum 전역 공유 상태 — Clone은 Arc 내부 참조만 복제
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt_secret: Arc<JwtSecret>,

    // ─── Services ────────────────────────────────────────────────────────────
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
    pub product_service: Arc<ProductService>,
    pub wallet_service: Arc<WalletService>,
    pub escrow_service: Arc<EscrowService>,
    pub chat_service: Arc<ChatService>,
    pub review_service: Arc<ReviewService>,
    pub report_service: Arc<ReportService>,
    pub admin_service: Arc<AdminService>,
    pub notification_service: Arc<NotificationService>,
}

impl AppState {
    /// AppState 생성 — main.rs에서 의존성 트리를 조립합니다.
    pub fn new(
        db: DatabaseConnection,
        jwt_secret: JwtSecret,
        auth_service: AuthService,
        user_service: UserService,
        product_service: ProductService,
        wallet_service: WalletService,
        escrow_service: EscrowService,
        chat_service: ChatService,
        review_service: ReviewService,
        report_service: ReportService,
        admin_service: AdminService,
        notification_service: NotificationService,
    ) -> Self {
        AppState {
            db,
            jwt_secret: Arc::new(jwt_secret),
            auth_service: Arc::new(auth_service),
            user_service: Arc::new(user_service),
            product_service: Arc::new(product_service),
            wallet_service: Arc::new(wallet_service),
            escrow_service: Arc::new(escrow_service),
            chat_service: Arc::new(chat_service),
            review_service: Arc::new(review_service),
            report_service: Arc::new(report_service),
            admin_service: Arc::new(admin_service),
            notification_service: Arc::new(notification_service),
        }
    }
}
