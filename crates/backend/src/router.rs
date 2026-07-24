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

use crate::{
    handlers::{
        admin_handler, auth_handler, chat_handler, escrow_handler, notification_handler,
        product_handler, report_handler, review_handler, upload_handler, user_handler,
        wallet_handler,
    },
    middleware::{auth::authenticate, rbac::require_admin},
    state::AppState,
};
use axum::{
    Router, middleware,
    routing::{get, patch, post, put},
};

/// 전체 애플리케이션 라우터 빌더
pub fn build_router(state: AppState) -> Router {
    // ─── 공개 라우트 (인증 불필요) ────────────────────────────────────────────
    let public_routes = Router::new()
        .route(
            "/auth/sendcode",
            post(auth_handler::sendcode).layer(middleware::from_fn_with_state(
                state.clone(),
                crate::middleware::limit::email_rate_limit_middleware,
            )),
        )
        .route("/auth/signup", post(auth_handler::signup))
        .route("/auth/login", post(auth_handler::login))
        .route("/products", get(product_handler::list_products))
        .route("/products/:item_uid", get(product_handler::get_product))
        .route("/users/:user_uid", get(user_handler::get_public_profile));

    // ─── 인증 필요 라우트 ────────────────────────────────────────────────────
    let protected_routes = Router::new()
        // Users
        .route(
            "/users/me",
            get(user_handler::get_my_profile).patch(user_handler::update_my_profile),
        )
        .route("/users/me/2fa/setup", post(auth_handler::setup_2fa))
        .route("/users/me/2fa/enable", post(auth_handler::enable_2fa))
        // Products
        .route("/products", post(product_handler::create_product))
        .route("/products/:item_uid", put(product_handler::update_product))
        .route(
            "/products/:item_uid/status",
            patch(product_handler::update_product_status),
        )
        .route(
            "/products/:item_uid/reports",
            post(report_handler::submit_report),
        )
        // Wallet
        .route("/wallet", get(wallet_handler::get_wallet))
        .route("/wallet/eth/withdraw", post(wallet_handler::eth_withdraw))
        .route("/wallet/history", get(wallet_handler::get_tx_history))
        // Escrow
        .route("/escrow", post(escrow_handler::initiate_escrow))
        .route("/escrow/:trade_uid", get(escrow_handler::get_escrow))
        .route(
            "/escrow/:trade_uid/deposit",
            post(escrow_handler::deposit_escrow),
        )
        .route(
            "/escrow/:trade_uid/confirm",
            post(escrow_handler::confirm_escrow),
        )
        .route(
            "/escrow/:trade_uid/dispute",
            post(escrow_handler::dispute_escrow),
        )
        .route(
            "/escrow/:trade_uid/reviews",
            post(review_handler::submit_review),
        )
        // Chat
        .route(
            "/chat/rooms",
            post(chat_handler::create_or_get_room).get(chat_handler::list_rooms),
        )
        .route(
            "/chat/rooms/:room_uid/history",
            get(chat_handler::get_chat_history),
        )
        .route(
            "/chat/rooms/:room_uid/messages",
            post(chat_handler::send_message),
        )
        .route("/chat/ws", get(chat_handler::websocket_handler))
        // Notifications
        .route(
            "/notifications",
            get(notification_handler::get_notifications),
        )
        .route(
            "/notifications/:noti_uid/read",
            patch(notification_handler::mark_notification_read),
        )
        // Uploads
        .route("/uploads", post(upload_handler::upload_image))
        .layer(axum::extract::DefaultBodyLimit::max(5 * 1024 * 1024))
        // JWT 인증 미들웨어 적용
        .layer(middleware::from_fn_with_state(state.clone(), authenticate));

    // ─── Admin 전용 라우트 (인증 + RBAC) ────────────────────────────────────
    let admin_routes = Router::new()
        .route("/admin/stats", get(admin_handler::get_platform_stats))
        .route(
            "/admin/escrow/force-settle",
            post(admin_handler::force_settle),
        )
        .route("/admin/users/:user_uid/ban", post(admin_handler::ban_user))
        .route(
            "/admin/products/:item_uid/hide",
            post(admin_handler::hide_product),
        )
        .route("/admin/reports", get(admin_handler::list_reports))
        // 1. Admin 역할 검사 (authenticate 이후)
        .layer(middleware::from_fn(require_admin))
        // 2. JWT 인증 (먼저 실행 — Axum은 레이어를 역순으로 실행)
        .layer(middleware::from_fn_with_state(state.clone(), authenticate));

    // ─── 최종 라우터 조립 ────────────────────────────────────────────────────
    use axum::http::header::{
        CONTENT_SECURITY_POLICY, HeaderValue, STRICT_TRANSPORT_SECURITY, X_CONTENT_TYPE_OPTIONS,
    };
    use tower_http::set_header::SetResponseHeaderLayer;

    use tower_http::services::{ServeDir, ServeFile};

    let static_dir = std::env::var("STATIC_DIR")
        .or_else(|_| std::env::var("LEPTOS_SITE_ROOT"))
        .unwrap_or_else(|_| "/app/dist".to_string());

    let index_html_path = format!("{}/index.html", static_dir);

    Router::new()
        .nest("/v1", public_routes)
        .nest("/v1", protected_routes)
        .nest("/v1", admin_routes)
        .nest_service("/uploads", ServeDir::new("uploads")).
        fallback_service(
            ServeDir::new(&static_dir)
                .not_found_service(ServeFile::new(index_html_path))
        )
        .with_state(state)
        // 글로벌 보안 헤더 추가
        .layer(SetResponseHeaderLayer::overriding(
            X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline';"),
        ))
}
