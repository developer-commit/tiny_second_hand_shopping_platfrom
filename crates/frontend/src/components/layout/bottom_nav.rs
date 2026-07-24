// crates/frontend/src/components/layout/bottom_nav.rs
// 목적: 모바일 하단 탭 네비게이션 — 홈/채팅/상품등록/지갑/마이페이지
// 네이티브 <a> 태그를 사용하여 class를 동적으로 설정합니다.
// Leptos 0.7에서 <A> 컴포넌트는 class prop을 지원하지 않으므로 <a>를 사용합니다.

use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn BottomNav() -> impl IntoView {
    let location = use_location();

    // 경로 매칭 헬퍼 클로저 반환
    let is_active = move |path: &'static str| {
        let current = location.pathname.get();
        if path == "/" {
            current == "/"
        } else {
            current.starts_with(path)
        }
    };

    view! {
        <nav class="bottom-nav" aria-label="하단 네비게이션">
            <a href="/"
               class=move || if is_active("/") { "bottom-nav-item active" } else { "bottom-nav-item" }
               aria-label="홈">
                <span class="bottom-nav-icon">"🏠"</span>
                <span class="bottom-nav-label">"홈"</span>
            </a>
            <a href="/chat"
               class=move || if is_active("/chat") { "bottom-nav-item active" } else { "bottom-nav-item" }
               aria-label="채팅">
                <span class="bottom-nav-icon">"💬"</span>
                <span class="bottom-nav-label">"채팅"</span>
            </a>
            <a href="/products/new"
               class=move || if is_active("/products/new") { "bottom-nav-item bottom-nav-center active" } else { "bottom-nav-item bottom-nav-center" }
               aria-label="상품 등록">
                <span class="bottom-nav-icon">"➕"</span>
                <span class="bottom-nav-label">"등록"</span>
            </a>
            <a href="/wallet"
               class=move || if is_active("/wallet") { "bottom-nav-item active" } else { "bottom-nav-item" }
               aria-label="지갑">
                <span class="bottom-nav-icon">"💰"</span>
                <span class="bottom-nav-label">"지갑"</span>
            </a>
            <a href="/mypage"
               class=move || if is_active("/mypage") { "bottom-nav-item active" } else { "bottom-nav-item" }
               aria-label="마이페이지">
                <span class="bottom-nav-icon">"👤"</span>
                <span class="bottom-nav-label">"마이"</span>
            </a>
        </nav>
    }
}
