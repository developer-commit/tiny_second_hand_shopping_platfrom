// crates/frontend/src/pages/home.rs
// URL: /
// 목적: 메인 페이지 — 상품 목록, 검색, 정렬 탭

use leptos::prelude::*;
use leptos_meta::Title;
use shared::dto::product_dto::ProductSearchQuery;
use crate::components::{
    layout::PageContainer,
    display::{ProductCard, EmptyState},
    input::SearchBar,
};
use crate::models::product_model::create_products_resource;

#[component]
pub fn HomePage() -> impl IntoView {
    let keyword = RwSignal::new(String::new());
    let search_query = RwSignal::new(String::new()); // To trigger search on enter

    let on_search = Callback::new(move |()| {
        search_query.set(keyword.get());
    });

    let products_res = create_products_resource(move || {
        let kw = search_query.get();
        ProductSearchQuery {
            keyword: if kw.is_empty() { None } else { Some(kw) },
            group_category: None,
            item_tag: None,
            page: Some(1),
            page_size: Some(20),
        }
    });

    view! {
        <Title text="홈"/>
        <PageContainer title="홈">
            <div class="flex flex-col gap-8">
                <section class="w-full">
                    <SearchBar
                        keyword=keyword
                        on_search=on_search.into()
                    />
                </section>
                
                <section class="w-full">
                    <Suspense fallback=move || view! { 
                        <div class="flex items-center justify-center py-20">
                            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-brand-600"></div>
                        </div> 
                    }>
                        {move || {
                            products_res.get().map(|res| {
                                match &*res {
                                    Ok(items) => {
                                        let items = items.clone();
                                        if items.is_empty() {
                                            view! {
                                                <div class="py-20">
                                                    <EmptyState
                                                        icon="🔍"
                                                        message="검색 결과가 없습니다."
                                                    />
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6">
                                                    {items.into_iter().map(|item| view! {
                                                        <ProductCard item=item />
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }
                                    }
                                    Err(err) => {
                                        view! {
                                            <div class="p-4 rounded-xl bg-red-50 text-red-600 border border-red-100 text-center">
                                                {format!("오류 발생: {}", err)}
                                            </div>
                                        }.into_any()
                                    }
                                }
                            })
                        }}
                    </Suspense>
                </section>
            </div>
        </PageContainer>
    }
}
