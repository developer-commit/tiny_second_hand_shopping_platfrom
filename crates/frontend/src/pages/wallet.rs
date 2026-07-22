// crates/frontend/src/pages/wallet.rs
// URL: /wallet, /wallet/withdraw

use leptos::prelude::*;
use leptos_meta::Title;
use shared::dto::transaction_dto::WithdrawReq;

#[component]
pub fn WalletPage() -> impl IntoView {
    // todo!("WalletUIState context 또는 Resource로 GET /v1/wallet 조회")
    let withdraw_address = RwSignal::new(String::new());
    let withdraw_amount = RwSignal::new(0.0_f64);
    let otp_token = RwSignal::new(String::new());
    let show_withdraw_modal = RwSignal::new(false);
    let estimated_fee = RwSignal::new(0.0_f64);

    let on_withdraw = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let req = WithdrawReq {
            destination_address: withdraw_address.get(),
            amount_bch: withdraw_amount.get(),
            otp_token: otp_token.get(),
        };
        todo!("POST /v1/wallet/withdraw → 출금 처리 → 내역 갱신")
    };

    view! {
        <Title text="지갑 대시보드"/>
        <main class="wallet-page">
            <h1>"지갑 대시보드"</h1>
            <section class="balance-section">
                // todo!("available_balance, locked_in_escrow 표시")
                <p>"잔액 조회 중..."</p>
            </section>
            <section class="address-section">
                // todo!("입금용 BCH 주소 표시 + QR 코드")
            </section>
            <button on:click=move |_| show_withdraw_modal.set(true)>"출금하기"</button>
            // 출금 모달 (2FA OTP 입력 포함)
            <Show when=move || show_withdraw_modal.get()>
                <div class="modal withdraw-modal">
                    <h2>"출금 신청"</h2>
                    <form on:submit=on_withdraw>
                        <input type="text" placeholder="수신 BCH 주소" on:input=move |e| withdraw_address.set(event_target_value(&e)) />
                        <input type="number" placeholder="출금 BCH" on:input=move |e| withdraw_amount.set(event_target_value(&e).parse().unwrap_or(0.0)) />
                        <p>"예상 수수료: " {move || estimated_fee.get()} " BCH"</p>
                        <input type="text" placeholder="OTP 코드 (2FA)" on:input=move |e| otp_token.set(event_target_value(&e)) />
                        <button type="submit">"출금 확인"</button>
                    </form>
                </div>
            </Show>
        </main>
    }
}
