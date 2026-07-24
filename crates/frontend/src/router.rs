// crates/frontend/src/router.rs
// 목적: Leptos 클라이언트 사이드 라우터.
// plan.md의 페이지 설계(2.3 웹페이지 설계)를 모두 반영합니다.

use crate::components::layout::AppShell;
use crate::pages::{
    admin::AdminPage, chat::ChatPage, escrow::EscrowPage, home::HomePage, login::LoginPage,
    mypage::MyPage, notifications::NotificationsPage, product_detail::ProductDetailPage,
    product_edit::ProductEditPage, product_new::ProductNewPage, public_profile::PublicProfilePage,
    signup::SignupPage, wallet::WalletPage,
};
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

#[component]
pub fn AppRouter() -> impl IntoView {
    view! {
        <Router>
            <AppShell>
                <Routes fallback=|| view! { <p>"404 — 페이지를 찾을 수 없습니다."</p> }>
                    // ─── 공개 라우트 ─────────────────────────────────────────────────────────
                    <Route path=path!("/") view=HomePage/>
                    <Route path=path!("/users/:user_uid") view=PublicProfilePage/>
                    <Route path=path!("/login") view=LoginPage/>
                    <Route path=path!("/signup") view=SignupPage/>
                    // ─── 인증 필요 라우트 ────────────────────────────────────────────────
                    <Route path=path!("/mypage") view=MyPage/>
                    <Route path=path!("/wallet") view=WalletPage/>
                    <Route path=path!("/wallet/withdraw") view=WalletPage/>
                    <Route path=path!("/products/new") view=ProductNewPage/>
                    <Route path=path!("/products/:item_uid") view=ProductDetailPage/>
                    <Route path=path!("/products/:item_uid/edit") view=ProductEditPage/>
                    <Route path=path!("/chat") view=ChatPage/>
                    <Route path=path!("/escrow/:trade_uid") view=EscrowPage/>
                    <Route path=path!("/notifications") view=NotificationsPage/>
                    // ─── 관리자 라우트 ──────────────────────────────────────────────────
                    <Route path=path!("/admin") view=AdminPage/>
                </Routes>
            </AppShell>
        </Router>
    }
}
