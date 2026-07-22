// crates/frontend/src/models/escrow_model.rs
// 목적: 에스크로 거래 상태 반응형 래퍼.

use leptos::prelude::*;
use shared::dto::escrow_dto::{SafeTradeStatusRes, TradeStep};

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
        let is_action_required =
            Signal::derive(move || step_sig.get() == TradeStep::Deposited);
        let is_disputed =
            Signal::derive(move || step_sig.get() == TradeStep::Disputed);
        let is_settled =
            Signal::derive(move || step_sig.get() == TradeStep::Settled);

        EscrowUIState {
            trade_uid: dto.trade_uid,
            locked_funds: RwSignal::new(dto.locked_funds),
            step: step_sig,
            auto_finalize_deadline: RwSignal::new(dto.auto_finalize_deadline),
            is_action_required,
            is_disputed,
            is_settled,
        }
    }
}
