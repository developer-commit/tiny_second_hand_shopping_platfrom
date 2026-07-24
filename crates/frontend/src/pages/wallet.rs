// crates/frontend/src/pages/wallet.rs
// URL: /wallet, /wallet/withdraw

use leptos::prelude::*;
use leptos_meta::Title;
use crate::components::{
    layout::PageContainer,
    display::{WalletBalanceCard, TxHistoryRow, EmptyState},
    input::{FormInput, PriceInput, OtpInput},
    feedback::{AppButton, ButtonVariant, Modal},
};
use crate::models::{
    wallet_model::{fetch_wallet_state, fetch_tx_history},
    auth_model::AuthStore,
};

#[component]
pub fn WalletPage() -> impl IntoView {
    let copy_success = RwSignal::new(false);

    let withdraw_address = RwSignal::new(String::new());
    let withdraw_amount = RwSignal::new(0.0_f64);
    let otp_token = RwSignal::new(String::new());
    let show_withdraw_modal = RwSignal::new(false);
    let estimated_fee = RwSignal::new(0.000_1_f64); // mock fee
    let withdraw_error = RwSignal::new(Option::<String>::None);
    let withdraw_success = RwSignal::new(false);

    let auth_store = expect_context::<AuthStore>();
    let token = Signal::derive(move || auth_store.bearer_header().unwrap_or_default());

    let wallet_res = LocalResource::new(move || {
        let t = token.get();
        async move {
            if t.is_empty() { return Err("로그인이 필요합니다.".to_string()); }
            fetch_wallet_state(&t).await
        }
    });

    let tx_res = LocalResource::new(move || {
        let t = token.get();
        async move {
            if t.is_empty() { return Err("로그인이 필요합니다.".to_string()); }
            fetch_tx_history(&t).await
        }
    });

    let eth_withdraw_action = Action::new_local(move |req: &shared::dto::transaction_dto::EthWithdrawReq| {
        let req_clone = req.clone();
        let t = token.get();
        async move {
            crate::models::wallet_model::eth_withdraw(&t, req_clone).await
        }
    });

    Effect::new(move |_| {
        if let Some(res) = eth_withdraw_action.value().get() {
            match res {
                Ok(_) => {
                    withdraw_success.set(true);
                    withdraw_error.set(None);
                    show_withdraw_modal.set(false);
                    // refresh resources
                    wallet_res.refetch();
                    tx_res.refetch();
                }
                Err(e) => {
                    withdraw_error.set(Some(e));
                }
            }
        }
    });

    let on_withdraw = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let req = shared::dto::transaction_dto::EthWithdrawReq {
            destination_eth: withdraw_address.get(),
            amount_eth: withdraw_amount.get(),
            otp_token: otp_token.get(),
        };
        eth_withdraw_action.dispatch(req);
    };

    view! {
        <Title text="내 지갑"/>
        <PageContainer title="내 지갑">
            <div class="wallet-page" style="max-width: 800px; margin: 0 auto; padding: 2rem; background-color: #f8f9fa; border: 2px solid #e9ecef; border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.05);">
                <h1 style="font-size: 1.75rem; font-weight: 700; color: #212529; margin-bottom: 1.5rem;">"내 지갑"</h1>
                
                <Suspense fallback=move || view! { <p style="padding: 2rem; text-align: center; color: #6c757d;">"지갑 정보를 불러오는 중..."</p> }>
                    {move || wallet_res.get().map(|res| match &*res {
                        Ok(state) => {
                            let state = state.clone();
                            let display_addr = if state.public_address.is_empty() {
                                "ETH 주소가 아직 생성되지 않았습니다.".to_string()
                            } else {
                                state.public_address.clone()
                            };
                                
                            view! {
                                <div style="display: flex; flex-direction: column; gap: 1.5rem; background: #ffffff; padding: 2rem; border-radius: 12px; border: 1px solid #dee2e6; box-shadow: 0 2px 4px rgba(0,0,0,0.02);">
                                    <WalletBalanceCard
                                        available=state.eth_balance
                                        locked=state.locked_in_escrow
                                    />
                                    <div style="background: #f8f9fa; padding: 1.5rem; border-radius: 8px; border: 1px solid #e9ecef;">
                                        <h3 style="margin-bottom: 1rem; color: #212529; font-weight: 600;">"ETH 입금 주소"</h3>
                                        <div style="display: flex; align-items: center; gap: 1rem;">
                                            <div style="flex: 1; word-break: break-all; font-family: 'Courier New', Courier, monospace; background: #e9ecef; padding: 1rem; border-radius: 6px; border: 1px solid #ced4da; color: #495057;">
                                                {display_addr.clone()}
                                            </div>
                                            <AppButton
                                                variant=ButtonVariant::Secondary
                                                loading=Signal::derive(|| false)
                                                disabled={
                                                    let addr = state.public_address.clone();
                                                    Signal::derive(move || addr.is_empty())
                                                }
                                                on_click={
                                                    let addr_click = state.public_address.clone();
                                                    move || {
                                                        if !addr_click.is_empty() {
                                                            let clean = addr_click.trim_start_matches("0x");
                                                            let hex_addr = format!("0x{}", clean.to_lowercase());
                                                            if let Some(window) = web_sys::window() {
                                                                let clipboard = window.navigator().clipboard();
                                                                let _ = clipboard.write_text(&hex_addr);
                                                                copy_success.set(true);
                                                                set_timeout(move || copy_success.set(false), std::time::Duration::from_millis(2000));
                                                            }
                                                        }
                                                    }
                                                }
                                            >
                                                {move || if copy_success.get() { "Copied!" } else { "복사" }}
                                            </AppButton>
                                        </div>
                                    </div>
                                    <div style="margin-top: 0.5rem;">
                                        <AppButton
                                            variant=ButtonVariant::Primary
                                            loading=Signal::derive(|| false)
                                            disabled=Signal::derive(|| false)
                                            on_click=move || show_withdraw_modal.set(true)
                                        >
                                            "ETH 출금하기"
                                        </AppButton>
                                    </div>
                                </div>
                            }.into_any()
                        },
                        Err(e) => view! { <div style="color: red;">{e.to_string()}</div> }.into_any(),
                    })}
                </Suspense>

                <div style="margin-top: 3rem; background: #ffffff; padding: 2rem; border-radius: 12px; border: 1px solid #dee2e6; box-shadow: 0 2px 4px rgba(0,0,0,0.02);">
                    <h2 style="font-size: 1.5rem; font-weight: 700; margin-bottom: 1.5rem; color: #212529; border-bottom: 2px solid #f8f9fa; padding-bottom: 1rem;">"거래 내역"</h2>
                    <Suspense fallback=move || view! { <p style="padding: 2rem; text-align: center; color: #6c757d;">"거래 내역 불러오는 중..."</p> }>
                        {move || tx_res.get().map(|res| match &*res {
                            Ok(history) => {
                                let history = history.clone();
                                if history.is_empty() {
                                    view! { 
                                        <div style="padding: 3rem; background: #f8f9fa; border-radius: 8px; border: 1px dashed #dee2e6;">
                                            <EmptyState icon="📝" message="거래 내역이 없습니다." />
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div style="display: flex; flex-direction: column; gap: 0.75rem;">
                                            {history.into_iter().map(|tx| view! {
                                                <div style="background: #f8f9fa; border: 1px solid #e9ecef; border-radius: 8px; padding: 1rem; transition: transform 0.2s; cursor: pointer; hover:shadow-md;">
                                                    <TxHistoryRow tx=tx />
                                                </div>
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                }
                            }
                            Err(e) => view! { <div style="color: red; padding: 1rem; background: #ffebee; border-radius: 8px;">{e.to_string()}</div> }.into_any(),
                        })}
                    </Suspense>
                </div>
            </div>

            <Modal is_open=show_withdraw_modal title="출금 신청".to_string()>
                <form on:submit=on_withdraw style="display: flex; flex-direction: column; gap: 1rem; padding-top: 1rem;">
                    <FormInput
                        label="받는 사람 주소".to_string()
                        placeholder="주소 입력".to_string()
                        input_type="text"
                        signal=withdraw_address
                        error=Signal::derive(|| None)
                    />
                    
                    <div>
                        <label style="display: block; margin-bottom: 0.5rem; font-size: 0.875rem;">"출금 수량"</label>
                        <PriceInput signal=withdraw_amount />
                        <p style="font-size: 0.75rem; color: var(--text-secondary); margin-top: 0.25rem;">
                            "예상 네트워크 수수료: " {move || estimated_fee.get()} " ETH"
                        </p>
                    </div>

                    <div>
                        <label style="display: block; margin-bottom: 0.5rem; font-size: 0.875rem;">"2FA 인증 코드 (6~8자리)"</label>
                        <OtpInput length=6 signal=otp_token />
                    </div>

                    {move || withdraw_error.get().map(|e| view! { <p class="error" style="color: red;">{e}</p> })}
                    
                    <AppButton
                        variant=ButtonVariant::Danger
                        loading=Signal::derive(move || eth_withdraw_action.pending().get())
                        disabled=Signal::derive(move || eth_withdraw_action.pending().get())
                        button_type="submit"
                    >
                        "출금 신청"
                    </AppButton>
                </form>
            </Modal>
        </PageContainer>
    }
}
