// crates/frontend/src/models/escrow_model.rs
// 목적: 에스크로 거래 상태 반응형 래퍼.

use gloo_net::http::Request;
use leptos::prelude::*;
use shared::dto::escrow_dto::{DisputeEscrowReq, InitiateEscrowReq, SafeTradeStatusRes, TradeStep};

const API_BASE_URL: &str = "/v1";

#[derive(Clone, Debug)]
pub struct EscrowUIState {
    pub trade_uid: String,
    pub locked_funds: RwSignal<f64>,
    pub step: RwSignal<TradeStep>,
    pub auto_finalize_deadline: RwSignal<Option<String>>,
    /// 구매자 액션(수령 확인/거부) 필요 여부 파생 시그널
    pub is_action_required: Signal<bool>,
    /// 분쟁 진행 중 여부
    pub is_disputed: Signal<bool>,
    /// 거래 완료 여부 (리뷰 작성 유도)
    pub is_settled: Signal<bool>,
}

impl EscrowUIState {
    pub fn from_dto(dto: SafeTradeStatusRes) -> Self {
        let step_sig = RwSignal::new(dto.step);
        let is_action_required = Signal::derive(move || step_sig.get() == TradeStep::Deposited);
        let is_disputed = Signal::derive(move || step_sig.get() == TradeStep::Disputed);
        let is_settled = Signal::derive(move || step_sig.get() == TradeStep::Settled);

        EscrowUIState {
            trade_uid: dto.trade_uid,
            locked_funds: RwSignal::new(dto.locked_funds.parse::<f64>().unwrap_or(0.0)),
            step: step_sig,
            auto_finalize_deadline: RwSignal::new(dto.auto_finalize_deadline),
            is_action_required,
            is_disputed,
            is_settled,
        }
    }
}

/// POST /escrow - 에스크로 예치 시작
pub async fn initiate_escrow(
    token: &str,
    req: InitiateEscrowReq,
) -> Result<SafeTradeStatusRes, String> {
    let url = format!("{}/escrow", API_BASE_URL);
    let res = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        let err_msg = err_res
            .map(|e| e.message)
            .unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
        return Err(err_msg);
    }

    res.json().await.map_err(|e| e.to_string())
}

/// POST /escrow/{trade_uid}/dispute - 수령 거부 (분쟁)
pub async fn dispute_escrow(
    token: &str,
    trade_uid: &str,
    req: DisputeEscrowReq,
) -> Result<SafeTradeStatusRes, String> {
    let url = format!("{}/escrow/{}/dispute", API_BASE_URL, trade_uid);
    let res = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        let err_msg = err_res
            .map(|e| e.message)
            .unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
        return Err(err_msg);
    }

    res.json().await.map_err(|e| e.to_string())
}

/// GET /escrow/{trade_uid} - 에스크로 상태 조회
pub async fn fetch_escrow_status(
    token: &str,
    trade_uid: &str,
) -> Result<SafeTradeStatusRes, String> {
    let url = format!("{}/escrow/{}", API_BASE_URL, trade_uid);
    let res = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        let err_msg = err_res
            .map(|e| e.message)
            .unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
        return Err(err_msg);
    }

    res.json().await.map_err(|e| e.to_string())
}

/// POST /escrow/{trade_uid}/deposit - 서버 지갑을 이용한 온체인 예치
pub async fn deposit_escrow(token: &str, trade_uid: &str) -> Result<(), String> {
    let url = format!("{}/escrow/{}/deposit", API_BASE_URL, trade_uid);
    let res = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        let err_msg = err_res
            .map(|e| e.message)
            .unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
        return Err(err_msg);
    }

    Ok(())
}
