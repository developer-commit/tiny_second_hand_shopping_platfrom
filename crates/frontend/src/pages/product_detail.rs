// crates/frontend/src/pages/product_detail.rs
// URL: /products/:item_uid

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

#[component]
pub fn ProductDetailPage() -> impl IntoView {
    let params = use_params_map();
    let item_uid = move || params.with(|p| p.get("item_uid").unwrap_or_default());

    let product = Resource::new(
        item_uid,
        |uid| async move {
            // todo!("GET /v1/products/{uid} → ItemDetailRes")
        },
    );

    view! {
        <Title text="상품 상세"/>
        <main class="product-detail">
            <Suspense fallback=move || view! { <p>"로딩 중..."</p> }>
                {move || product.get().map(|_item| view! {
                    <div class="product-info">
                        // todo!("heading, asking_price, current_state, seller 신뢰도 표시")
                        <p>"상품 정보"</p>
                        // 구매/채팅 버튼 (is_available에 따라 분기)
                        <div class="action-buttons">
                            <button>"채팅하기"</button>
                            <button>"안전결제"</button>
                        </div>
                        // 신고하기 폼
                        <details>
                            <summary>"신고하기"</summary>
                            // todo!("SubmitReportReq 폼 → POST /v1/products/{item_uid}/reports")
                        </details>
                    </div>
                })}
            </Suspense>
        </main>
    }
}
