// crates/frontend/src/pages/product_new.rs
// URL: /products/new

use leptos::prelude::*;
use leptos_meta::Title;
use shared::dto::product_dto::CreateItemReq;

#[component]
pub fn ProductNewPage() -> impl IntoView {
    let heading = RwSignal::new(String::new());
    let detail_body = RwSignal::new(String::new());
    let asking_price = RwSignal::new(0.0_f64);
    let group_category = RwSignal::new(String::new());
    let tags_input = RwSignal::new(String::new());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let tags: Vec<String> = tags_input.get().split(',').map(|t| t.trim().to_string()).collect();
        let req = CreateItemReq {
            heading: heading.get(),
            detail_body: detail_body.get(),
            asking_price: asking_price.get(),
            group_category: group_category.get(),
            item_tags: tags,
            image_urls: vec![],
        };
        todo!("POST /v1/products → 성공 시 /products/<item_uid>으로 이동")
    };

    view! {
        <Title text="상품 등록"/>
        <main class="product-new">
            <h1>"상품 등록"</h1>
            <form on:submit=on_submit>
                <input type="text" placeholder="상품명" on:input=move |e| heading.set(event_target_value(&e)) />
                <textarea placeholder="상품 설명" on:input=move |e| detail_body.set(event_target_value(&e))></textarea>
                <input type="number" placeholder="가격 (BCH)" on:input=move |e| asking_price.set(event_target_value(&e).parse().unwrap_or(0.0)) />
                <input type="text" placeholder="카테고리" on:input=move |e| group_category.set(event_target_value(&e)) />
                <input type="text" placeholder="태그 (쉼표 구분)" on:input=move |e| tags_input.set(event_target_value(&e)) />
                <button type="submit">"등록하기"</button>
            </form>
        </main>
    }
}
