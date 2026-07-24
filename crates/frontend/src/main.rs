// crates/frontend/src/main.rs
// 목적: Leptos 앱 마운트 진입점.
// AuthStore, NotificationStore, ToastStore를 Context Provider로 전체 앱에 주입합니다.

mod components;
mod models;
mod pages;
mod router;

use leptos::prelude::*;
use leptos_meta::provide_meta_context;
use crate::{
    components::feedback::{ToastContainer, ToastStore},
    models::{
        auth_model::AuthStore,
        notification_store::NotificationStore,
        chat_store::ChatStore,
    },
    router::AppRouter,
};

#[component]
fn App() -> impl IntoView {
    provide_meta_context();

    // ─── 전역 상태 Context 주입 ──────────────────────────────────────────────
    provide_context(AuthStore::new());
    provide_context(NotificationStore::new());
    provide_context(ToastStore::new());
    provide_context(ChatStore::new());

    view! {
        // 전역 토스트 알림 컨테이너 (화면 우하단 고정)
        <ToastContainer/>
        <AppRouter/>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App)
}
