// crates/frontend/src/models/wallet_model.rs
// 목적: 지갑 상태 반응형 래퍼.

use leptos::prelude::*;
use shared::dto::transaction_dto::{TxHistoryItemRes, WalletStateRes, WithdrawReq, EthWithdrawReq};
use gloo_net::http::Request;

const API_BASE_URL: &str = "/v1";

#[derive(Clone, Debug)]
pub struct WalletUIState {
    pub public_address: RwSignal<String>,
    pub available_balance: RwSignal<f64>,
    pub locked_in_escrow: RwSignal<f64>,
    pub eth_balance: RwSignal<f64>,
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
            eth_balance: RwSignal::new(0.0),
            withdrawable,
            tx_history: RwSignal::new(Vec::new()),
            is_loading: RwSignal::new(false),
        }
    }

    pub fn update_from_dto(&self, dto: WalletStateRes) {
        self.public_address.set(dto.public_address);
        self.available_balance.set(dto.available_balance);
        self.locked_in_escrow.set(dto.locked_in_escrow);
        self.eth_balance.set(dto.eth_balance);
    }
}

/// GET /wallet - 지갑 상태 조회
pub async fn fetch_wallet_state(token: &str) -> Result<WalletStateRes, String> {
    let url = format!("{}/wallet", API_BASE_URL);
    let res = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        let err_msg = err_res.map(|e| e.message).unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
        return Err(err_msg);
    }

    res.json().await.map_err(|e| e.to_string())
}

/// GET /wallet/history - 트랜잭션 내역 조회
pub async fn fetch_tx_history(token: &str) -> Result<Vec<TxHistoryItemRes>, String> {
    let url = format!("{}/wallet/history", API_BASE_URL);
    let res = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        let err_msg = err_res.map(|e| e.message).unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
        return Err(err_msg);
    }

    res.json().await.map_err(|e| e.to_string())
}

/// POST /wallet/withdraw - 출금 요청
pub async fn withdraw(token: &str, req: WithdrawReq) -> Result<(), String> {
    let url = format!("{}/wallet/withdraw", API_BASE_URL);
    let res = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        let err_msg = err_res.map(|e| e.message).unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
        return Err(err_msg);
    }

    Ok(())
}

/// POST /wallet/eth/withdraw - ETH 출금 요청
pub async fn eth_withdraw(token: &str, req: EthWithdrawReq) -> Result<(), String> {
    let url = format!("{}/wallet/eth/withdraw", API_BASE_URL);
    let res = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        let err_msg = err_res.map(|e| e.message).unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
        return Err(err_msg);
    }

    Ok(())
}

/// BTC Dummy logic
pub fn btc_withdraw_dummy() {
    if let Some(window) = web_sys::window() {
        let _ = window.alert_with_message("BTC 출금은 아직 준비 중입니다 (Coming Soon).");
    }
}
