// crates/frontend/src/pages/escrow.rs
// URL: /escrow/:trade_uid

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;
use shared::dto::escrow_dto::DisputeEscrowReq;
use shared::dto::review_dto::SubmitReviewReq;

#[component]
pub fn EscrowPage() -> impl IntoView {
    let params = use_params_map();
    let trade_uid = move || params.with(|p| p.get("trade_uid").unwrap_or_default());
    let dispute_reason = RwSignal::new(String::new());
    let review_score = RwSignal::new(5_i32);
    let review_feedback = RwSignal::new(String::new());

    view! {
        <Title text="안전결제"/>
        <main class="escrow-page">
            <h1>"안전결제 진행 상황"</h1>
            // todo!("EscrowUIState 로드 → step에 따라 UI 분기")
            <section class="actions">
                // 수령 승인
                <button on:click=move |_| { todo!("POST /v1/escrow/{}/confirm", trade_uid()) }>"수령 확인"</button>
                // 수령 거부 (분쟁)
                <div class="dispute-form">
                    <textarea placeholder="거부 사유" on:input=move |e| dispute_reason.set(event_target_value(&e))></textarea>
                    <button on:click=move |_| {
                        let req = DisputeEscrowReq { cause: dispute_reason.get() };
                        todo!("POST /v1/escrow/{}/dispute", trade_uid())
                    }>"수령 거부"</button>
                </div>
            </section>
            // 거래 완료 후 리뷰 작성 폼
            // todo!("is_settled Signal 구독 → 완료 시 리뷰 폼 표시")
            <section class="review-form">
                <h2>"거래 후기 작성"</h2>
                <input type="number" min="1" max="5" on:input=move |e| review_score.set(event_target_value(&e).parse().unwrap_or(5)) />
                <textarea placeholder="후기 내용" on:input=move |e| review_feedback.set(event_target_value(&e))></textarea>
                <button on:click=move |_| {
                    let req = SubmitReviewReq {
                        score: review_score.get(),
                        feedback: Some(review_feedback.get()),
                    };
                    todo!("POST /v1/escrow/{}/reviews", trade_uid())
                }>"후기 제출"</button>
            </section>
        </main>
    }
}
