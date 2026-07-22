// crates/backend/src/router.rs
// 목적: 전체 Axum 라우터 빌더.
//
// [보안 구조]
// - 인증이 필요한 라우트: authenticate 미들웨어 레이어
// - Admin 전용 라우트: authenticate + require_admin 레이어 (이중 보호)
// - 공개 라우트: 미들웨어 없음 (상품 목록, 로그인, 회원가입)
//
// [라우터 구조]
// /v1
// ├── auth/ (공개)
// │   ├── POST /signup
// │   └── POST /login
// ├── users/ (인증)
// │   └── me/ ...
// ├── products/ (조회 공개, 등록/수정 인증)
// ├── wallet/ (인증 + 2FA)
// ├── escrow/ (인증)
// ├── chat/ (인증 + WS)
// ├── notifications/ (인증)
// └── admin/ (인증 + RBAC::Admin)

use axum::{middleware, Router, routing::{get, patch, post}};
use crate::{
    handlers::{
        auth_handler,
        user_handler,
        product_handler,
        wallet_handler,
        escrow_handler,
        chat_handler,
        review_handler,
        report_handler,
        admin_handler,
        notification_handler,
    },
    middleware::{auth::authenticate, rbac::require_admin},
    state::AppState,
};

/// 전체 애플리케이션 라우터 빌더
pub fn build_router(state: AppState) -> Router {
    // ─── 공개 라우트 (인증 불필요) ────────────────────────────────────────────
    let public_routes = Router::new()
        .route("/auth/signup", post(auth_handler::signup))
        .route("/auth/login", post(auth_handler::login))
        .route("/products", get(product_handler::list_products))
        .route("/products/:item_uid", get(product_handler::get_product));

    // ─── 인증 필요 라우트 ────────────────────────────────────────────────────
    let protected_routes = Router::new()
        // Users
        .route("/users/me", get(user_handler::get_my_profile).patch(user_handler::update_my_profile))
        .route("/users/me/2fa/setup", post(auth_handler::setup_2fa))
        .route("/users/me/2fa/enable", post(auth_handler::enable_2fa))
        // Products
        .route("/products", post(product_handler::create_product))
        .route("/products/:item_uid/status", patch(product_handler::update_product_status))
        .route("/products/:item_uid/reports", post(report_handler::submit_report))
        // Wallet
        .route("/wallet", get(wallet_handler::get_wallet))
        .route("/wallet/withdraw", post(wallet_handler::withdraw))
        .route("/wallet/history", get(wallet_handler::get_tx_history))
        // Escrow
        .route("/escrow", post(escrow_handler::initiate_escrow))
        .route("/escrow/:trade_uid/confirm", post(escrow_handler::confirm_escrow))
        .route("/escrow/:trade_uid/dispute", post(escrow_handler::dispute_escrow))
        .route("/escrow/:trade_uid/reviews", post(review_handler::submit_review))
        // Chat
        .route("/chat/rooms", post(chat_handler::create_or_get_room))
        .route("/chat/rooms/:room_uid/history", get(chat_handler::get_chat_history))
        .route("/chat/ws", get(chat_handler::websocket_handler))
        // Notifications
        .route("/notifications", get(notification_handler::get_notifications))
        .route("/notifications/:noti_uid/read", patch(notification_handler::mark_notification_read))
        // JWT 인증 미들웨어 적용
        .layer(middleware::from_fn_with_state(state.clone(), authenticate));

    // ─── Admin 전용 라우트 (인증 + RBAC) ────────────────────────────────────
    let admin_routes = Router::new()
        .route("/admin/stats", get(admin_handler::get_platform_stats))
        .route("/admin/escrow/force-settle", post(admin_handler::force_settle))
        // 1. Admin 역할 검사 (authenticate 이후)
        .layer(middleware::from_fn(require_admin))
        // 2. JWT 인증 (먼저 실행 — Axum은 레이어를 역순으로 실행)
        .layer(middleware::from_fn_with_state(state.clone(), authenticate));

    // ─── 최종 라우터 조립 ────────────────────────────────────────────────────
    Router::new()
        .nest("/v1", public_routes)
        .nest("/v1", protected_routes)
        .nest("/v1", admin_routes)
        .with_state(state)
}
