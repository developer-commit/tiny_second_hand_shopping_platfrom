// crates/frontend/src/components/display/trade_step_badge.rs
// 목적: 에스크로 거래 단계 배지
// TradeStep Enum 각 단계를 색상+레이블로 매핑합니다.

use leptos::prelude::*;
use shared::dto::escrow_dto::TradeStep;

#[component]
pub fn TradeStepBadge(step: TradeStep) -> impl IntoView {
    let (icon, label, class) = match step {
        TradeStep::PendingDeposit => ("⏳", "입금 대기", "badge badge-warning"),
        TradeStep::Deposited => ("🛡️", "안전결제",   "badge badge-primary"),
        TradeStep::Received  => ("📦", "수령확인",   "badge badge-success"),
        TradeStep::Disputed  => ("⚠️", "분쟁중",     "badge badge-danger"),
        TradeStep::Settled   => ("✅", "정산완료",   "badge badge-secondary"),
        TradeStep::Refunded  => ("↩️", "환불완료",   "badge badge-muted"),
    };

    view! {
        <span class=class>
            <span>{icon}</span>
            " "
            <span>{label}</span>
        </span>
    }
}
