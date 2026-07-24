// crates/frontend/src/components/layout/app_shell.rs
// 목적: 전체 레이아웃 래퍼 — <NavBar/> + <main> + <BottomNav/>
// 모든 페이지는 이 컴포넌트 안에 렌더링됩니다.

use leptos::prelude::*;
use super::{NavBar, BottomNav};

#[component]
pub fn AppShell(children: Children) -> impl IntoView {
    view! {
        <div class="min-h-screen flex flex-col bg-slate-50 text-slate-900 font-sans">
            <NavBar/>
            <main class="flex-1 w-full pb-20">
                {children()}
            </main>
            <BottomNav/>
        </div>
    }
}
