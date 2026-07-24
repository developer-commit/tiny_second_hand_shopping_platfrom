// crates/frontend/src/pages/product_detail.rs
// URL: /products/:item_uid

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;
use shared::dto::product_dto::ItemState;
use crate::components::{
    layout::PageContainer,
    display::{StatusBadge, ImageCarousel, UserTrustIndicator},
    feedback::{AppButton, ButtonVariant},
    input::form_input::FormInput,
};
use crate::models::{
    product_model::fetch_product_detail,
    auth_model::AuthStore,
};
use shared::dto::{
    chat_dto::{CreateChatRoomReq, ChatRoomRes},
    escrow_dto::{InitiateEscrowReq, SafeTradeStatusRes},
    report_dto::SubmitReportReq,
};
use gloo_net::http::Request;
use leptos_router::hooks::use_navigate;

async fn create_chat_room(token: &str, req: &CreateChatRoomReq) -> Result<ChatRoomRes, String> {
    let url = format!("{}/chat/rooms", "/v1");
    let res = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        if let Ok(api_err) = err_res {
            return Err(api_err.message);
        }
        return Err("채팅방 생성 실패".into());
    }
    res.json().await.map_err(|e| e.to_string())
}

async fn initiate_escrow(token: &str, req: &InitiateEscrowReq) -> Result<SafeTradeStatusRes, String> {
    let url = format!("{}/escrow", "/v1");
    let res = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        if let Ok(api_err) = err_res {
            return Err(api_err.message);
        }
        return Err("에스크로 생성 실패".into());
    }
    res.json().await.map_err(|e| e.to_string())
}

async fn submit_report(token: &str, item_uid: &str, req: &SubmitReportReq) -> Result<(), String> {
    let url = format!("{}/products/{}/reports", "/v1", item_uid);
    let res = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        return Err("신고 접수 실패".into());
    }
    Ok(())
}

#[component]
pub fn ProductDetailPage() -> impl IntoView {
    let params = use_params_map();
    let item_uid = move || params.with(|p| p.get("item_uid").unwrap_or_default());
    let auth_store = expect_context::<AuthStore>();
    let token = Signal::derive(move || auth_store.bearer_header().unwrap_or_default());
    let current_user_uid = Signal::derive(move || auth_store.current_user.get().map(|u| u.user_uid));
    let navigate = use_navigate();

    let product = LocalResource::new(move || {
        let uid = item_uid();
        async move {
            if uid.is_empty() { return Err("유효하지 않은 상품입니다.".to_string()); }
            fetch_product_detail(uid).await
        }
    });

    let chat_error = RwSignal::new(Option::<String>::None);
    let chat_action = Action::new_local(move |req: &CreateChatRoomReq| {
        let t = token.get();
        let req_clone = req.clone();
        async move {
            chat_error.set(None);
            if t.is_empty() { return Err("로그인이 필요합니다.".to_string()); }
            create_chat_room(&t, &req_clone).await
        }
    });

    let escrow_error = RwSignal::new(Option::<String>::None);
    let escrow_action = Action::new_local(move |req: &InitiateEscrowReq| {
        let t = token.get();
        let req_clone = req.clone();
        async move {
            escrow_error.set(None);
            if t.is_empty() { return Err("로그인이 필요합니다.".to_string()); }
            initiate_escrow(&t, &req_clone).await
        }
    });

    let report_cause = RwSignal::new(String::new());
    let report_action = Action::new_local(move |req: &SubmitReportReq| {
        let t = token.get();
        let uid = item_uid();
        let req_clone = req.clone();
        async move {
            if t.is_empty() { return Err("로그인이 필요합니다.".to_string()); }
            submit_report(&t, &uid, &req_clone).await
        }
    });

    let nav1 = navigate.clone();
    Effect::new(move |_| {
        if let Some(res) = chat_action.value().get() {
            match res {
                Ok(room) => nav1(&format!("/chat?room_uid={}", room.room_uid), Default::default()),
                Err(e) => chat_error.set(Some(e)),
            }
        }
    });

    let nav2 = navigate.clone();
    Effect::new(move |_| {
        if let Some(res) = escrow_action.value().get() {
            match res {
                Ok(trade) => nav2(&format!("/escrow/{}", trade.trade_uid), Default::default()),
                Err(e) => escrow_error.set(Some(e)),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(_)) = report_action.value().get() {
            report_cause.set(String::new());
        }
    });

    view! {
        <Title text="상품 상세"/>
        <PageContainer title="상품 상세">
            <div class="product-detail" style="max-width: 800px; margin: 0 auto; padding-top: 2rem;">
                <Suspense fallback=move || view! { <p>"상품 정보를 불러오는 중..."</p> }>
                    {move || product.get().map(|res| match &*res {
                        Ok(item) => {
                            let item = item.clone();
                            let is_available = matches!(item.current_state, ItemState::OnSale);
                            let is_owner = current_user_uid.get() == Some(item.owner_uid.clone());
                            
                            view! {
                                <div style="display: flex; flex-direction: column; gap: 2rem;">
                                    // Images
                                    <div style="aspect-ratio: 16/9; background: var(--surface-color); border-radius: 8px; overflow: hidden;">
                                        <ImageCarousel urls=item.image_urls.clone() />
                                    </div>
                                    
                                    // Header Info
                                    <div style="display: flex; flex-direction: column; gap: 1rem;">
                                        <div style="display: flex; align-items: center; justify-content: space-between;">
                                            <StatusBadge state=item.current_state />
                                            <span style="font-size: 0.875rem; color: var(--text-secondary);">
                                                {format!("조회 {}", item.hit_count)}
                                            </span>
                                        </div>
                                        
                                        <h1 style="font-size: 1.5rem; font-weight: bold;">{item.heading.clone()}</h1>
                                        <div style="font-size: 1.5rem; font-weight: bold; color: var(--primary-color);">
                                            {
                                                let currency_str = "ETH";
                                                format!("{} {}", item.asking_price, currency_str)
                                            }
                                        </div>
                                    </div>
                                    
                                    // Seller Info
                                    <div style="display: flex; justify-content: space-between; align-items: center; padding: 1rem; border-top: 1px solid var(--border-color); border-bottom: 1px solid var(--border-color);">
                                        <span style="font-weight: 500;">{item.owner_uid.clone()}</span>
                                        <UserTrustIndicator reliability_index=item.seller_trust_score.unwrap_or(0.0) />
                                    </div>
                                    
                                    // Body
                                    <p style="white-space: pre-wrap; line-height: 1.6;">
                                        {item.detail_body.clone()}
                                    </p>
                                    
                                    // Action Buttons
                                    <div style="display: flex; gap: 1rem; margin-top: 1rem;">
                                        <div style="flex: 1;">
                                            <AppButton
                                                variant=ButtonVariant::Secondary
                                                loading=chat_action.pending()
                                                disabled=Signal::derive(move || !is_available || is_owner)
                                                on_click={
                                                    let uid = item.item_uid.clone();
                                                    let owner_uid = item.owner_uid.clone();
                                                    move || { 
                                                        if is_owner {
                                                            chat_error.set(Some("자신의 상품에는 채팅을 시작할 수 없습니다.".to_string()));
                                                            return;
                                                        }
                                                        chat_action.dispatch(CreateChatRoomReq { item_uid: uid.clone(), partner_uid: owner_uid.clone() }); 
                                                    }
                                                }
                                            >
                                                "채팅하기"
                                            </AppButton>
                                        </div>
                                        <div style="flex: 1;">
                                            <AppButton
                                                variant=ButtonVariant::Primary
                                                loading=escrow_action.pending()
                                                on_click={
                                                    let uid = item.item_uid.clone();
                                                    move || { escrow_action.dispatch(InitiateEscrowReq { item_uid: uid.clone() }); }
                                                }
                                            >
                                                {if is_available { "안전결제 (에스크로)" } else { "에스크로 현황 보기" }}
                                            </AppButton>
                                        </div>
                                    </div>
                                    {move || chat_error.get().map(|e| view! {
                                        <div style="color: var(--danger-color, red); background: #ffebee; padding: 0.75rem; border-radius: 8px; margin-top: 1rem; text-align: center; font-size: 0.9rem;">
                                            {e}
                                        </div>
                                    })}
                                    {move || escrow_error.get().map(|e| view! {
                                        <div style="color: var(--danger-color, red); background: #ffebee; padding: 0.75rem; border-radius: 8px; margin-top: 0.5rem; text-align: center; font-size: 0.9rem;">
                                            {e}
                                        </div>
                                    })}
                                    
                                    // Report Form
                                    <details style="margin-top: 2rem; border-top: 1px solid var(--border-color); padding-top: 1rem;">
                                        <summary style="cursor: pointer; color: var(--text-secondary);">"이 게시글 신고하기"</summary>
                                        <div style="margin-top: 1rem; padding: 1rem; background: var(--surface-color); border-radius: 8px;">
                                            <form on:submit=move |ev: leptos::ev::SubmitEvent| {
                                                ev.prevent_default();
                                                report_action.dispatch(SubmitReportReq { cause: report_cause.get() });
                                            } style="display: flex; flex-direction: column; gap: 1rem;">
                                                <FormInput label="신고 사유" placeholder="자세한 신고 사유를 입력해주세요" input_type="text" signal=report_cause error=Signal::derive(|| None) />
                                                <AppButton variant=ButtonVariant::Danger loading=report_action.pending() disabled=Signal::derive(move || report_cause.get().is_empty()) button_type="submit">
                                                    "신고 접수"
                                                </AppButton>
                                            </form>
                                        </div>
                                    </details>
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! { <div style="color: red;">{e.to_string()}</div> }.into_any(),
                    })}
                </Suspense>
            </div>
        </PageContainer>
    }
}
