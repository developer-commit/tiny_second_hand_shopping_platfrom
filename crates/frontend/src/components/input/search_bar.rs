// crates/frontend/src/components/input/search_bar.rs
// 목적: 검색창 — 돋보기 아이콘 + 입력 필드
// 부모가 on_search 콜백을 제공하며, Enter 키 또는 버튼 클릭으로 트리거합니다.

use leptos::prelude::*;

#[component]
pub fn SearchBar(
    keyword: RwSignal<String>,
    on_search: Callback<()>,
) -> impl IntoView {
    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" {
            on_search.run(());
        }
    };

    view! {
        <div class="relative max-w-2xl w-full mx-auto group">
            <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                <svg class="w-5 h-5 text-slate-400 group-focus-within:text-brand-500 transition-colors" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
            </div>
            <input
                type="search"
                class="block w-full pl-11 pr-24 py-3.5 bg-white border border-slate-200 rounded-2xl text-slate-900 placeholder-slate-400 shadow-sm focus:outline-none focus:ring-2 focus:ring-brand-500/20 focus:border-brand-500 transition-all text-base"
                placeholder="어떤 상품을 찾고 계신가요?"
                prop:value=move || keyword.get()
                on:input=move |ev| keyword.set(event_target_value(&ev))
                on:keydown=on_keydown
                aria-label="상품 검색"
            />
            <div class="absolute inset-y-0 right-2 flex items-center">
                <button
                    type="button"
                    class="px-4 py-2 bg-brand-600 hover:bg-brand-700 text-white text-sm font-medium rounded-xl shadow-sm hover:shadow transition-colors"
                    on:click=move |_| on_search.run(())
                    aria-label="검색 실행"
                >
                    "검색"
                </button>
            </div>
        </div>
    }
}
