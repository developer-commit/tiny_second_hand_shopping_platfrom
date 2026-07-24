// crates/frontend/src/pages/escrow.rs
// URL: /escrow/:trade_uid

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::components::{
    display::{StepProgress, TradeStepBadge},
    feedback::{AppButton, ButtonVariant, Modal},
    input::{FormTextarea, StarRating},
    layout::PageContainer,
};
use crate::models::{
    auth_model::AuthStore,
    escrow_model::{deposit_escrow, dispute_escrow, fetch_escrow_status},
    wallet_model::fetch_wallet_state,
};
use gloo_net::http::Request;
use shared::dto::escrow_dto::{DisputeEscrowReq, TradeStep};
use shared::dto::review_dto::SubmitReviewReq;

async fn confirm_escrow(token: &str, trade_uid: &str) -> Result<(), String> {
    let url = format!("{}/escrow/{}/confirm", "/v1", trade_uid);
    let res = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        return Err("구매 확정 실패".into());
    }
    Ok(())
}

async fn submit_review(token: &str, trade_uid: &str, req: &SubmitReviewReq) -> Result<(), String> {
    let url = format!("{}/escrow/{}/reviews", "/v1", trade_uid);
    let res = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        return Err("리뷰 등록 실패".into());
    }
    Ok(())
}

#[component]
pub fn EscrowPage() -> impl IntoView {
    let params = use_params_map();
    let trade_uid = move || params.with(|p| p.get("trade_uid").unwrap_or_default());

    let auth_store = expect_context::<AuthStore>();
    let token = Signal::derive(move || auth_store.bearer_header().unwrap_or_default());
    let current_user_uid = Signal::derive(move || {
        auth_store
            .current_user
            .get()
            .map(|u| u.user_uid)
            .unwrap_or_default()
    });

    let dispute_reason = RwSignal::new(String::new());
    let show_dispute_modal = RwSignal::new(false);
    let review_score = RwSignal::new(5_i32);
    let review_feedback = RwSignal::new(String::new());
    let dispute_error = RwSignal::new(Option::<String>::None);

    let escrow_res = LocalResource::new(move || {
        let t = token.get();
        let uid = trade_uid();
        async move {
            if t.is_empty() || uid.is_empty() {
                return Err("유효하지 않은 요청입니다.".to_string());
            }
            fetch_escrow_status(&t, &uid).await
        }
    });

    let wallet_res = LocalResource::new(move || {
        let t = token.get();
        async move {
            if t.is_empty() {
                return Err("로그인이 필요합니다.".to_string());
            }
            fetch_wallet_state(&t).await
        }
    });

    let dispute_action = Action::new_local(move |req: &DisputeEscrowReq| {
        let req_clone = req.clone();
        let t = token.get();
        let uid = trade_uid();
        async move {
            if t.is_empty() || uid.is_empty() {
                return Err("유효하지 않은 요청입니다.".to_string());
            }
            dispute_escrow(&t, &uid, req_clone).await
        }
    });

    let confirm_action = Action::new_local(move |_: &()| {
        let t = token.get();
        let uid = trade_uid();
        async move {
            if t.is_empty() || uid.is_empty() {
                return Err("유효하지 않은 요청입니다.".to_string());
            }
            confirm_escrow(&t, &uid).await
        }
    });

    let review_action = Action::new_local(move |req: &SubmitReviewReq| {
        let req_clone = req.clone();
        let t = token.get();
        let uid = trade_uid();
        async move {
            if t.is_empty() || uid.is_empty() {
                return Err("유효하지 않은 요청입니다.".to_string());
            }
            submit_review(&t, &uid, &req_clone).await
        }
    });

    let deposit_error = RwSignal::new(Option::<String>::None);
    let is_depositing = RwSignal::new(false);

    let deposit_action = Action::new_local(move |_: &()| {
        let t = token.get();
        let uid = trade_uid();
        async move {
            if t.is_empty() || uid.is_empty() {
                return Err("유효하지 않은 요청입니다.".to_string());
            }
            is_depositing.set(true);
            deposit_error.set(None);
            let res = deposit_escrow(&t, &uid).await;
            is_depositing.set(false);
            res
        }
    });

    Effect::new(move |_| {
        if let Some(res) = dispute_action.value().get() {
            match res {
                Ok(_) => {
                    show_dispute_modal.set(false);
                    escrow_res.refetch();
                }
                Err(e) => {
                    dispute_error.set(Some(e));
                }
            }
        }
    });

    let confirm_error = RwSignal::new(Option::<String>::None);
    Effect::new(move |_| {
        if let Some(Ok(_)) = confirm_action.value().get() {
            escrow_res.refetch();
            wallet_res.refetch();
            confirm_error.set(None);
        } else if let Some(Err(e)) = confirm_action.value().get() {
            confirm_error.set(Some(e.clone()));
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(_)) = review_action.value().get() {
            // Can optionally show success toast here
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(_)) = deposit_action.value().get() {
            escrow_res.refetch();
            wallet_res.refetch();
        } else if let Some(Err(e)) = deposit_action.value().get() {
            deposit_error.set(Some(e.clone()));
        }
    });

    view! {
        <Title text="안전결제"/>
        <PageContainer title="안전결제">
            <div class="escrow-page" style="max-width: 600px; margin: 0 auto; padding-top: 2rem; display: flex; flex-direction: column; gap: 2rem;">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <h1 style="font-size: 1.5rem; font-weight: bold;">"안전결제 진행 상황"</h1>
                    <Suspense fallback=move || view! { <span>"지갑 정보 불러오는 중..."</span> }>
                        {move || wallet_res.get().map(|res| match &*res {
                            Ok(wallet) => view! {
                                <div style="font-size: 0.9rem; background: var(--brand-50, #f0f9ff); padding: 0.5rem 1rem; border-radius: 9999px; border: 1px solid var(--brand-200, #bae6fd); color: var(--brand-700, #0369a1); font-weight: 500;">
                                    {format!("내 잔액: {:.4} ETH", wallet.eth_balance)}
                                </div>
                            }.into_any(),
                            Err(_) => view! { <span></span> }.into_any(),
                        })}
                    </Suspense>
                </div>

                <Suspense fallback=move || view! { <p>"상태를 불러오는 중..."</p> }>
                    {move || escrow_res.get().map(|res| match &*res {
                        Ok(state) => {
                            let current_step = match state.step {
                                TradeStep::PendingDeposit => 0,
                                TradeStep::Deposited => 1,
                                TradeStep::Received => 2,
                                TradeStep::Disputed => 0,
                                TradeStep::Settled => 4,
                                TradeStep::Refunded => 0,
                            };
                            let is_pending_deposit = matches!(state.step, TradeStep::PendingDeposit);
                            let is_settled = matches!(state.step, TradeStep::Settled);
                            let is_seller = current_user_uid.get() == state.seller_uid;
                            let is_buyer = current_user_uid.get() == state.buyer_uid;
                            let buyer_exists = !state.buyer_uid.is_empty();

                            let trade_step = state.step.clone();
                            let seller_uid = state.seller_uid.clone();
                            let buyer_uid = state.buyer_uid.clone();


                            let progress_steps = vec![
                                "입금/결제 대기".to_string(),
                                "결제 완료".to_string(),
                                "물품 수령 확인".to_string(),
                                "정산 완료".to_string()
                            ];

                            view! {
                                <div style="display: flex; flex-direction: column; gap: 2rem;">
                                    <div style="background: var(--surface-color); padding: 1.5rem; border-radius: 8px;">
                                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem;">
                                            <span style="font-weight: 500;">{format!("주문번호: {}", trade_uid())}</span>
                                            <TradeStepBadge step=trade_step />
                                        </div>
                                        <div style="margin-bottom: 1.5rem; font-size: 0.9rem; color: var(--text-secondary);">
                                            <p>"판매자: " {seller_uid.clone()}</p>
                                            <p>"구매자: " {if buyer_exists { buyer_uid.clone() } else { "구매자 대기 중".to_string() }}</p>
                                        </div>
                                        <StepProgress steps=progress_steps current=current_step />
                                    </div>

                                    <Show when=move || !is_settled && !is_pending_deposit>
                                        <Show when=move || is_buyer>
                                            {move || confirm_error.get().map(|e| view! { <div style="color: red; margin-bottom: 1rem; font-size: 0.9rem;">{e}</div> })}
                                            <section class="actions" style="display: flex; gap: 1rem;">
                                                <div style="flex: 1;">
                                                    <AppButton
                                                        variant=ButtonVariant::Primary
                                                        loading=confirm_action.pending()
                                                        disabled=Signal::derive(move || current_step != 1 && current_step != 2 || confirm_action.pending().get())
                                                        on_click=move || { confirm_action.dispatch(()); }
                                                    >
                                                        "수령 확인 (결제 승인)"
                                                    </AppButton>
                                                </div>
                                                <div style="flex: 1;">
                                                    <AppButton
                                                        variant=ButtonVariant::Danger
                                                        loading=Signal::derive(|| false)
                                                        disabled=Signal::derive(move || current_step != 1 && current_step != 2)
                                                        on_click=move || show_dispute_modal.set(true)
                                                    >
                                                        "수령 거부 (분쟁 신청)"
                                                    </AppButton>
                                                </div>
                                            </section>
                                        </Show>
                                        <Show when=move || is_seller>
                                            <div style="background: var(--surface-color); padding: 1rem; border-radius: 8px; text-align: center; border: 1px solid #ccc;">
                                                {move || {
                                                    if !buyer_exists || current_step == 0 {
                                                        "Waiting for Buyer/Deposit"
                                                    } else {
                                                        "Awaiting Delivery Confirmation"
                                                    }
                                                }}
                                            </div>
                                        </Show>
                                    </Show>

                                    <Show when=move || is_pending_deposit>
                                        <Show when=move || is_buyer>
                                            <div style="background: var(--surface-color); padding: 1.5rem; border-radius: 8px; border: 1px solid var(--border-color);">
                                                <h3 style="font-weight: bold; margin-bottom: 1rem;">"결제 진행 (서버 지갑)"</h3>
                                                <p style="margin-bottom: 1rem; color: var(--text-secondary);">
                                                    "에스크로 안전결제를 위해 시스템이 자동으로 스마트 컨트랙트에 예치금을 입금합니다."
                                                </p>

                                                {move || deposit_error.get().map(|e| view! { <div style="color: red; margin-bottom: 1rem; font-size: 0.9rem;">{e}</div> })}

                                                <AppButton
                                                    variant=ButtonVariant::Primary
                                                    loading=Signal::derive(move || is_depositing.get() || deposit_action.pending().get())
                                                    disabled=Signal::derive(move || is_depositing.get() || deposit_action.pending().get())
                                                    on_click=move || {
                                                        deposit_action.dispatch(());
                                                    }
                                                >
                                                    {move || if deposit_action.pending().get() {
                                                        "온체인 예치 진행 중..."
                                                    } else {
                                                        "안전결제 진행하기"
                                                    }}
                                                </AppButton>
                                            </div>
                                        </Show>
                                        <Show when=move || is_seller>
                                            <div style="background: var(--surface-color); padding: 1rem; border-radius: 8px; text-align: center; border: 1px solid #ccc;">
                                                "구매자가 스마트 컨트랙트에 결제 대금을 예치하기를 기다리고 있습니다."
                                            </div>
                                        </Show>
                                    </Show>

                                    <Show when=move || is_settled>
                                        <div style="background: var(--success-color, #d4edda); color: var(--success-text, #155724); padding: 1rem; border-radius: 8px; text-align: center; font-weight: bold; border: 1px solid #c3e6cb;">
                                            "✅ 결제가 성공적으로 승인되었습니다. 거래가 완료되었습니다."
                                        </div>
                                        <section class="review-form" style="background: var(--surface-color); padding: 1.5rem; border-radius: 8px;">
                                            <h2 style="font-size: 1.25rem; font-weight: bold; margin-bottom: 1rem;">"거래 후기 작성"</h2>
                                            <div style="display: flex; flex-direction: column; gap: 1rem;">
                                                <div style="display: flex; justify-content: center;">
                                                    <StarRating score=review_score />
                                                </div>
                                                <FormTextarea
                                                    label="후기 내용"
                                                    placeholder="후기 내용을 입력해주세요"
                                                    signal=review_feedback
                                                    error=Signal::derive(|| None)
                                                />
                                                <AppButton
                                                    variant=ButtonVariant::Primary
                                                    loading=review_action.pending()
                                                    disabled=Signal::derive(move || review_action.pending().get() || review_feedback.get().is_empty())
                                                    on_click=move || {
                                                        review_action.dispatch(SubmitReviewReq {
                                                            score: review_score.get(),
                                                            feedback: Some(review_feedback.get()),
                                                        });
                                                    }
                                                >
                                                    "후기 제출"
                                                </AppButton>
                                            </div>
                                        </section>
                                    </Show>
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! { <div style="color: red;">{e.to_string()}</div> }.into_any(),
                    })}
                </Suspense>
            </div>

            <Modal is_open=show_dispute_modal title="분쟁 신청">
                <div style="display: flex; flex-direction: column; gap: 1rem; padding-top: 1rem;">
                    <p style="color: var(--text-secondary); font-size: 0.875rem;">
                        "수령 거부 사유를 자세히 입력해주세요. 운영진이 확인 후 조치합니다."
                    </p>
                    <FormTextarea
                        label="거부 사유"
                        placeholder="거부 사유"
                        signal=dispute_reason
                        error=Signal::derive(|| None)
                    />
                    {move || dispute_error.get().map(|e| view! { <p class="error" style="color: red;">{e.clone()}</p> })}
                    <div style="display: flex; gap: 1rem; margin-top: 1rem;">
                        <div style="flex: 1;">
                            <AppButton
                                variant=ButtonVariant::Ghost
                                loading=Signal::derive(|| false)
                                disabled=Signal::derive(|| false)
                                on_click=move || show_dispute_modal.set(false)
                            >
                                "취소"
                            </AppButton>
                        </div>
                        <div style="flex: 1;">
                            <AppButton
                                variant=ButtonVariant::Danger
                                loading=dispute_action.pending()
                                disabled=Signal::derive(move || dispute_reason.get().is_empty())
                                on_click=move || {
                                    dispute_action.dispatch(DisputeEscrowReq {
                                        cause: dispute_reason.get()
                                    });
                                }
                            >
                                "신청하기"
                            </AppButton>
                        </div>
                    </div>
                </div>
            </Modal>
        </PageContainer>
    }
}
