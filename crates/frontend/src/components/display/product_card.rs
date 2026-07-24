// crates/frontend/src/components/display/product_card.rs
// 목적: 홈 상품 그리드 카드 — 썸네일, 제목, 가격, 조회수, 상태 배지
// <a> 네이티브 태그를 사용하여 class를 직접 설정합니다.

use super::StatusBadge;
use leptos::prelude::*;
use shared::dto::product_dto::ItemSummaryRes;

#[component]
pub fn ProductCard(item: ItemSummaryRes) -> impl IntoView {
    let href = format!("/products/{}", item.item_uid);
    let currency_str = "ETH";
    let price_str = format!("{:.4} {}", item.asking_price, currency_str);
    let thumbnail = item.thumbnail_url.clone().unwrap_or_default();
    let has_thumbnail = !thumbnail.is_empty();
    let heading = item.heading.clone();
    let hit_count = item.hit_count;

    view! {
        <a href=href class="group flex flex-col bg-white rounded-2xl overflow-hidden shadow-sm hover:shadow-xl hover:-translate-y-1 transition-all duration-300 border border-slate-100">
            // ── 썸네일 ────────────────────────────────────────────
            <div class="relative aspect-[4/3] bg-slate-100 overflow-hidden">
                <Show
                    when=move || has_thumbnail
                    fallback=|| view! {
                        <div class="absolute inset-0 flex items-center justify-center text-4xl text-slate-300">
                            <span>"📦"</span>
                        </div>
                    }
                >
                    <img
                        src=thumbnail.clone()
                        alt="상품 이미지"
                        class="absolute inset-0 w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
                        loading="lazy"
                    />
                </Show>
                <div class="absolute top-3 left-3 z-10">
                    <StatusBadge state=item.current_state.clone()/>
                </div>
            </div>

            // ── 정보 영역 ─────────────────────────────────────────
            <div class="p-4 flex flex-col gap-1.5 flex-1">
                <h3 class="text-slate-900 font-medium leading-snug line-clamp-2 group-hover:text-brand-600 transition-colors">{heading}</h3>
                <div class="mt-auto pt-2 flex items-center justify-between">
                    <p class="text-lg font-bold text-slate-900">{price_str}</p>
                    <p class="text-xs font-medium text-slate-400 flex items-center gap-1 bg-slate-50 px-2 py-1 rounded-md">
                        <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                        </svg>
                        <span>{hit_count}</span>
                    </p>
                </div>
            </div>
        </a>
    }
}
