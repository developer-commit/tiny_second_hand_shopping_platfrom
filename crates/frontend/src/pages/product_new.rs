// crates/frontend/src/pages/product_new.rs
// URL: /products/new

use crate::components::{
    feedback::{AppButton, ButtonVariant},
    input::{CategorySelect, FormInput, FormTextarea, ImageUploader, PriceInput, TagInput},
    layout::PageContainer,
};
use crate::models::auth_model::AuthStore;
use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;
use shared::dto::product_dto::CreateItemReq;

#[component]
pub fn ProductNewPage() -> impl IntoView {
    let heading = RwSignal::new(String::new());
    let detail_body = RwSignal::new(String::new());
    let asking_price = RwSignal::new(0.0_f64);
    let currency = RwSignal::new(shared::dto::common_dto::Currency::ETH);
    let group_category = RwSignal::new(String::new());
    let tags_input = RwSignal::new(Vec::<String>::new());
    let image_files = RwSignal::new(Vec::<String>::new());
    let error_msg = RwSignal::new(Option::<String>::None);

    let navigate = use_navigate();
    let auth_store = expect_context::<AuthStore>();
    let token = Signal::derive(move || auth_store.bearer_header().unwrap_or_default());

    let submit_action = Action::new_local(move |req: &CreateItemReq| {
        let req_clone = req.clone();
        let t = token.get();
        async move {
            if t.is_empty() {
                return Err("로그인이 필요합니다.".to_string());
            }

            let req_with_images = req_clone;

            let res = Request::post("/v1/products")
                .header("Authorization", &format!("Bearer {}", t))
                .json(&req_with_images)
                .map_err(|e| e.to_string())?
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !res.ok() {
                let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
                let err_msg = err_res
                    .map(|e| e.message)
                    .unwrap_or_else(|_| "Failed to create product".to_string());
                return Err(err_msg);
            }

            // Assume the API returns ItemSummaryRes or similar with item_uid
            // For now just return ok and let it redirect to home or somewhere
            Ok(())
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let req = CreateItemReq {
            heading: heading.get(),
            detail_body: detail_body.get(),
            asking_price: asking_price.get(),
            currency: currency.get(),
            group_category: group_category.get(),
            item_tags: tags_input.get(),
            image_urls: image_files.get(),
        };
        submit_action.dispatch(req);
    };

    Effect::new(move |_| {
        if let Some(res) = submit_action.value().get() {
            match res {
                Ok(_) => {
                    navigate("/", Default::default());
                }
                Err(e) => {
                    error_msg.set(Some(e));
                }
            }
        }
    });

    view! {
        <Title text="상품 등록"/>
        <PageContainer title="상품 등록">
            <div class="product-new" style="max-width: 600px; margin: 0 auto; padding-top: 2rem;">
                <h1 style="font-size: 1.5rem; font-weight: bold; margin-bottom: 2rem;">"상품 등록"</h1>

                <form on:submit=on_submit style="display: flex; flex-direction: column; gap: 1.5rem;">
                    <ImageUploader preview_urls=image_files on_uploaded=Callback::new(move |_| {}) />

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
                        <label style="display: block; margin-bottom: 0.5rem; font-size: 0.875rem;">"통화"</label>
                        <select
                            disabled
                            style="width: 100%; padding: 0.75rem; border-radius: 0.5rem; border: 1px solid #d1d5db; outline: none; transition: border-color 0.2s; background-color: #f3f4f6;"
                        >
                            <option value="ETH" selected>"ETH"</option>
                        </select>
                    </div>

                    <div>
                        <label style="display: block; margin-bottom: 0.5rem; font-size: 0.875rem;">
                            "가격 ("
                            "ETH"
                            ")"
                        </label>
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

                    {move || error_msg.get().map(|e| view! { <p class="error" style="color: red;">{e}</p> })}

                    <AppButton
                        variant=ButtonVariant::Primary
                        loading=submit_action.pending()
                        disabled=submit_action.pending()
                        button_type="submit"
                    >
                        "등록하기"
                    </AppButton>
                </form>
            </div>
        </PageContainer>
    }
}
