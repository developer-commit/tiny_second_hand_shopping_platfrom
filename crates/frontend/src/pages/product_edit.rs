// crates/frontend/src/pages/product_edit.rs
// URL: /products/:item_uid/edit

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

#[component]
pub fn ProductEditPage() -> impl IntoView {
    let params = use_params_map();
    let item_uid = move || params.with(|p| p.get("item_uid").unwrap_or_default());
    let current_state_input = RwSignal::new(String::new());

    view! {
        <Title text="상품 수정"/>
        <main class="product-edit">
            <h1>"상품 수정"</h1>
            // todo!("기존 정보 로드 → 수정 폼 → PUT /v1/products/{item_uid}")
            <section class="state-change">
                <h2>"상품 상태 변경"</h2>
                <select on:change=move |e| current_state_input.set(event_target_value(&e))>
                    <option value="on_sale">"판매중"</option>
                    <option value="reserved">"예약중"</option>
                    <option value="sold">"판매완료"</option>
                </select>
                <button on:click=move |_| { todo!("PATCH /v1/products/{}/status", item_uid()) }>"상태 변경"</button>
            </section>
        </main>
    }
}
