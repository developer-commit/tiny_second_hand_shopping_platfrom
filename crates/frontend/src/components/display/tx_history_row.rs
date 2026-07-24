// crates/frontend/src/components/display/tx_history_row.rs
// 목적: 지갑 거래 내역 행 — MovementType별 아이콘 + 금액 + 상태

use leptos::prelude::*;
use shared::dto::transaction_dto::{MovementType, TxHistoryItemRes, TxStatus};

#[component]
pub fn TxHistoryRow(tx: TxHistoryItemRes) -> impl IntoView {
    let (icon, label, amount_class) = match tx.movement_type {
        MovementType::Deposit => ("⬇️", "입금", "tx-amount-positive"),
        MovementType::Withdrawal => ("⬆️", "출금", "tx-amount-negative"),
        MovementType::EscrowLock => ("🔒", "에스크로 잠금", "tx-amount-neutral"),
        MovementType::EscrowRelease => ("🔓", "에스크로 해제", "tx-amount-positive"),
    };

    let status_label = match tx.process_status {
        TxStatus::Pending => "처리중",
        TxStatus::Confirmed => "완료",
        TxStatus::Failed => "실패",
    };
    let status_class = match tx.process_status {
        TxStatus::Pending => "tx-status tx-status-pending",
        TxStatus::Confirmed => "tx-status tx-status-confirmed",
        TxStatus::Failed => "tx-status tx-status-failed",
    };

    let amount_str = format!("{:.4} ETH", tx.amount);
    let timestamp = tx.timestamp.clone();

    view! {
        <div class="tx-history-row">
            <span class="tx-icon">{icon}</span>
            <div class="tx-info">
                <span class="tx-label">{label}</span>
                <span class="tx-time">{timestamp}</span>
            </div>
            <div class="tx-right">
                <span class=amount_class>{amount_str}</span>
                <span class=status_class>{status_label}</span>
            </div>
        </div>
    }
}
