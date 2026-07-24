// crates/frontend/src/components/layout/navbar.rs
// 목적: 상단 글로벌 네비게이션 바
// AuthStore · NotificationStore를 use_context로 구독합니다.
// 네이티브 <a> 태그를 사용하여 class를 설정합니다.

use leptos::prelude::*;
use crate::models::auth_model::AuthStore;
use crate::models::notification_store::NotificationStore;

#[component]
pub fn NavBar() -> impl IntoView {
    let auth_store = use_context::<AuthStore>()
        .expect("AuthStore must be provided");
    let noti_store = use_context::<NotificationStore>()
        .expect("NotificationStore must be provided");

    let is_authenticated = auth_store.is_authenticated;
    let has_unread = noti_store.has_unread;
    let display_name = move || {
        auth_store.current_user.get()
            .map(|u| u.display_name.clone())
            .unwrap_or_default()
    };
    let avatar_char = move || {
        display_name().chars().next()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "U".to_string())
    };

    view! {
        <header class="sticky top-0 z-50 w-full backdrop-blur-md bg-white/80 border-b border-slate-200/80 shadow-sm transition-all">
            <div class="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
                // ── 로고 ──────────────────────────────────────────
                <a href="/" class="flex items-center gap-2 group">
                    <span class="flex items-center justify-center w-8 h-8 rounded-lg bg-brand-600 text-white font-bold text-lg shadow-sm group-hover:bg-brand-700 transition-colors">"₿"</span>
                    <span class="font-bold text-xl tracking-tight text-slate-900">"ETH 마켓"</span>
                </a>

                // ── 우측 액션 영역 ────────────────────────────────
                <nav class="flex items-center gap-4">
                    <Show
                        when=move || is_authenticated.get()
                        fallback=|| view! {
                            <a href="/login" class="text-sm font-medium text-slate-600 hover:text-slate-900 transition-colors">"로그인"</a>
                            <a href="/signup" class="px-4 py-2 text-sm font-medium text-white bg-brand-600 hover:bg-brand-700 rounded-full shadow-sm hover:shadow-md transition-all">"회원가입"</a>
                        }
                    >
                        // 알림 버튼
                        <a href="/notifications" class="relative p-2 text-slate-500 hover:text-brand-600 transition-colors bg-slate-100 hover:bg-slate-200 rounded-full" aria-label="알림">
                            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                            </svg>
                            <Show when=move || has_unread.get()>
                                <span class="absolute top-1 right-1.5 w-2.5 h-2.5 bg-red-500 border-2 border-white rounded-full" aria-label="읽지 않은 알림"></span>
                            </Show>
                        </a>
                        // 프로필
                        <a href="/mypage" class="flex items-center gap-2 p-1 pl-1.5 pr-3 bg-white border border-slate-200 rounded-full hover:border-brand-300 hover:shadow-sm transition-all" aria-label="마이페이지">
                            <span class="flex items-center justify-center w-7 h-7 bg-brand-100 text-brand-700 font-semibold rounded-full text-xs">
                                {avatar_char}
                            </span>
                            <span class="text-sm font-medium text-slate-700 max-w-[80px] truncate">{display_name}</span>
                        </a>
                    </Show>
                </nav>
            </div>
        </header>
    }
}
