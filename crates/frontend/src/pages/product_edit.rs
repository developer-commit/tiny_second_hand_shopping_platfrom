// crates/frontend/src/pages/product_edit.rs
// URL: /products/:item_uid/edit

use crate::components::{
    feedback::{AppButton, ButtonVariant},
    input::{CategorySelect, FormInput, FormTextarea, PriceInput, TagInput},
    layout::PageContainer,
};
use crate::models::auth_model::AuthStore;
use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_navigate, use_params_map};
use shared::dto::product_dto::{ItemState, UpdateItemReq, UpdateItemStateReq};

async fn update_item_status(
    token: &str,
    item_uid: &str,
    req: &UpdateItemStateReq,
) -> Result<(), String> {
    let url = format!("{}/products/{}/status", "/v1", item_uid);
    let res = Request::patch(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        return Err("상태 변경 실패".into());
    }
    Ok(())
}

async fn update_item(token: &str, item_uid: &str, req: &UpdateItemReq) -> Result<(), String> {
    let url = format!("{}/products/{}", "/v1", item_uid);
    let res = Request::put(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        return Err("상품 수정 실패".into());
    }
    Ok(())
}

#[component]
pub fn ProductEditPage() -> impl IntoView {
    let params = use_params_map();
    let item_uid = move || params.with(|p| p.get("item_uid").unwrap_or_default());
    let current_state_input = RwSignal::new(String::new());

    let _navigate = use_navigate();

    let heading = RwSignal::new(String::new());
    let detail_body = RwSignal::new(String::new());
    let asking_price = RwSignal::new(0.0_f64);
    let group_category = RwSignal::new(String::new());
    let tags_input = RwSignal::new(Vec::<String>::new());

    let auth_store = expect_context::<AuthStore>();
    let token = Signal::derive(move || auth_store.bearer_header().unwrap_or_default());

    let status_action = Action::new_local(move |req: &UpdateItemStateReq| {
        let t = token.get();
        let uid = item_uid();
        let req_clone = req.clone();
        async move { update_item_status(&t, &uid, &req_clone).await }
    });

    let update_action = Action::new_local(move |req: &UpdateItemReq| {
        let t = token.get();
        let uid = item_uid();
        let req_clone = req.clone();
        async move { update_item(&t, &uid, &req_clone).await }
    });

    Effect::new(move |_| {
        if let Some(Ok(_)) = update_action.value().get() {
            // Can show a toast or redirect
        }
    });

    view! {
        <Title text="상품 수정"/>
        <PageContainer title="상품 수정">
            <div class="product-edit" style="max-width: 600px; margin: 0 auto; padding-top: 2rem;">
                <h1 style="font-size: 1.5rem; font-weight: bold; margin-bottom: 2rem;">"상품 수정"</h1>

                <section class="state-change" style="background: var(--surface-color); padding: 1.5rem; border-radius: 8px; margin-bottom: 2rem;">
                    <h2 style="font-size: 1.25rem; font-weight: bold; margin-bottom: 1rem;">"상품 상태 변경"</h2>
                    <div style="display: flex; gap: 1rem; align-items: flex-end;">
                        <div style="flex: 1;">
                            <label style="display: block; margin-bottom: 0.5rem; font-size: 0.875rem;">"상태"</label>
                            <select
                                on:change=move |e| current_state_input.set(event_target_value(&e))
                                style="width: 100%; padding: 0.75rem; border-radius: 4px; border: 1px solid var(--border-color); background: var(--bg-color);"
                            >
                                <option value="on_sale">"판매중"</option>
                                <option value="reserved">"예약중"</option>
                                <option value="sold">"판매완료"</option>
                            </select>
                        </div>
                        <AppButton
                            variant=ButtonVariant::Secondary
                            loading=status_action.pending()
                            disabled=status_action.pending()
                            on_click=move || {
                                let state = match current_state_input.get().as_str() {
                                    "reserved" => ItemState::Reserved,
                                    "sold" => ItemState::Sold,
                                    _ => ItemState::OnSale,
                                };
                                status_action.dispatch(UpdateItemStateReq { current_state: state });
                            }
                        >
                            "상태 변경"
                        </AppButton>
                    </div>
                </section>

                <form on:submit=move |ev| {
                    ev.prevent_default();
                    update_action.dispatch(UpdateItemReq {
                        heading: Some(heading.get()),
                        detail_body: Some(detail_body.get()),
                        asking_price: Some(asking_price.get()),
                        group_category: Some(group_category.get()),
                        item_tags: Some(tags_input.get()),
                    });
                } style="display: flex; flex-direction: column; gap: 1.5rem;">

                    <FormInput
                        label="상품명"
                        placeholder="상품의 제목을 입력하세요"
                        input_type="text"
                        signal=heading
                        error=Signal::derive(|| None)
                    />

                    <div>
                        <label style="display: block; margin-bottom: 0.5rem; font-size: 0.875rem;">"카테고리"</label>
                        <CategorySelect signal=group_category />
                    </div>

                    <div>
                        <label style="display: block; margin-bottom: 0.5rem; font-size: 0.875rem;">"가격"</label>
                        <PriceInput signal=asking_price />
                    </div>

                    <FormTextarea
                        label="상품 설명"
                        placeholder="상품의 상세 설명을 입력하세요"
                        signal=detail_body
                        error=Signal::derive(|| None)
                    />

                    <div>
                        <label style="display: block; margin-bottom: 0.5rem; font-size: 0.875rem;">"태그"</label>
                        <TagInput tags=tags_input />
                    </div>

                    <AppButton
                        variant=ButtonVariant::Primary
                        loading=update_action.pending()
                        disabled=update_action.pending()
                        button_type="submit"
                    >
                        "수정하기"
                    </AppButton>
                </form>
            </div>
        </PageContainer>
    }
}
