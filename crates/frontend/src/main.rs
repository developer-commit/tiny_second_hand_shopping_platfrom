// crates/frontend/src/main.rs
// 목적: Leptos 앱 마운트 진입점.
// AuthStore, NotificationStore를 Context Provider로 전체 앱에 주입합니다.

mod models;
mod pages;
mod router;

use leptos::prelude::*;
use leptos_meta::provide_meta_context;
use crate::{
    models::{
        auth_model::AuthStore,
        notification_store::NotificationStore,
    },
    router::AppRouter,
};

#[component]
fn App() -> impl IntoView {
    provide_meta_context();

    // ─── 전역 상태 Context 주입 ──────────────────────────────────────────────
    provide_context(AuthStore::new());
    provide_context(NotificationStore::new());

    view! {
        <AppRouter/>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App)
}
