// crates/frontend/src/pages/home.rs
// URL: /
// 목적: 메인 페이지 — 상품 목록, 검색, 정렬 탭

use leptos::prelude::*;
use leptos_meta::Title;
use shared::dto::product_dto::ItemSummaryRes;

#[component]
pub fn HomePage() -> impl IntoView {
    let keyword = RwSignal::new(String::new());
    let category = RwSignal::new(Option::<String>::None);

    let products = Resource::new(
        move || (keyword.get(), category.get()),
        |(_kw, _cat)| async move {
            // todo!("GET /v1/products?keyword=...&group_category=... API 호출 → Vec<ItemSummaryRes>")
            Vec::<ItemSummaryRes>::new()
        },
    );

    view! {
        <Title text="중고거래 플랫폼 — 메인"/>
        <main class="home-page">
            <section class="search-bar">
                <input
                    type="text"
                    placeholder="상품 검색..."
                    on:input=move |e| keyword.set(event_target_value(&e))
                />
            </section>
            <section class="product-grid">
                <Suspense fallback=move || view! { <p>"로딩 중..."</p> }>
                    {move || products.get().map(|items| {
                        items.into_iter().map(|item| view! {
                            <div class="product-card">
                                <a href=format!("/products/{}", item.item_uid)>
                                    <span class="heading">{item.heading}</span>
                                    <span class="price">{item.asking_price} " BCH"</span>
                                    <span class="hit-count">{item.hit_count} " 회 조회"</span>
                                </a>
                            </div>
                        }).collect_view()
                    })}
                </Suspense>
            </section>
        </main>
    }
}
