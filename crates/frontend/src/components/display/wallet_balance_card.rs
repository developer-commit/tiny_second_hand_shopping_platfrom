// crates/frontend/src/components/display/wallet_balance_card.rs
// 목적: 지갑 잔액 카드 — 가용 잔액 / 에스크로 잠금 / 출금 가능 3분할 표시

use leptos::prelude::*;

#[component]
pub fn WalletBalanceCard(available: f64, locked: f64) -> impl IntoView {
    let withdrawable = available - locked;

    view! {
        <div class="wallet-balance-card">
            <div class="wallet-balance-grid">
                <div class="wallet-balance-item">
                    <span class="wallet-balance-label">"가용 잔액"</span>
                    <span class="wallet-balance-value">{format!("{:.4} ETH", available)}</span>
                </div>
                <div class="wallet-balance-item wallet-balance-locked">
                    <span class="wallet-balance-label">"에스크로 잠금"</span>
                    <span class="wallet-balance-value">{format!("{:.4} ETH", locked)}</span>
                </div>
            </div>
            <div class="wallet-balance-withdrawable">
                <span class="wallet-balance-label">"출금 가능"</span>
                <span class="wallet-balance-value wallet-balance-value-primary">
                    {format!("{:.4} ETH", withdrawable.max(0.0))}
                </span>
            </div>
        </div>
    }
}
