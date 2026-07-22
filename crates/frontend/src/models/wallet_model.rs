// crates/frontend/src/models/wallet_model.rs
// 목적: 지갑 상태 반응형 래퍼.

use leptos::prelude::*;
use shared::dto::transaction_dto::{TxHistoryItemRes, WalletStateRes};

#[derive(Clone, Debug)]
pub struct WalletUIState {
    pub public_address: RwSignal<String>,
    pub available_balance: RwSignal<f64>,
    pub locked_in_escrow: RwSignal<f64>,
    /// 출금 가능 잔액 파생 시그널 (available - locked)
    pub withdrawable: Signal<f64>,
    pub tx_history: RwSignal<Vec<TxHistoryItemRes>>,
    pub is_loading: RwSignal<bool>,
}

impl WalletUIState {
    pub fn new() -> Self {
        let available = RwSignal::new(0.0_f64);
        let locked = RwSignal::new(0.0_f64);
        let withdrawable = Signal::derive(move || available.get() - locked.get());

        WalletUIState {
            public_address: RwSignal::new(String::new()),
            available_balance: available,
            locked_in_escrow: locked,
            withdrawable,
            tx_history: RwSignal::new(Vec::new()),
            is_loading: RwSignal::new(false),
        }
    }

    pub fn update_from_dto(&self, dto: WalletStateRes) {
        self.public_address.set(dto.public_address);
        self.available_balance.set(dto.available_balance);
        self.locked_in_escrow.set(dto.locked_in_escrow);
    }
}
