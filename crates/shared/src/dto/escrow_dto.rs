// crates/shared/src/dto/escrow_dto.rs
// 목적: 에스크로 안전결제 관련 Request/Response DTO.
// 스키마 은닉:
// - amount → locked_funds, status → step
// - auto_confirm_at → auto_finalize_deadline, dispute_reason → cause
// 보안: trade_uid, item_uid 모두 OpaqueId로 난독화

use super::common_dto::Currency;
use crate::types::OpaqueId;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// 에스크로 진행 단계를 Enum으로 타입화하여 유효하지 않은 상태 전달 차단
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TradeStep {
    PendingDeposit, // DB: "pending_deposit" — 예치 대기 중 (클라이언트 트랜잭션 필요)
    Deposited,      // DB: "deposited" — 대금 예치
    Received,       // DB: "received" — 구매자 수령 승인
    Disputed,       // DB: "disputed" — 분쟁 발생
    Settled,        // DB: "settled" — 정산 완료
    Refunded,       // DB: "refunded" — 환불 완료
}

/// [Request] POST /escrow — 에스크로 예치 시작
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct InitiateEscrowReq {
    pub item_uid: OpaqueId, // DB: product_id (난독화)
}

/// [Request] POST /escrow/{trade_uid}/dispute — 수령 거부 (분쟁)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DisputeEscrowReq {
    #[validate(length(min = 10, max = 2000))]
    pub cause: String, // DB: dispute_reason
}

/// [Response] 에스크로 거래 상태
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeTradeStatusRes {
    pub trade_uid: OpaqueId,                    // DB: id (난독화)
    pub item_uid: OpaqueId,                     // DB: product_id (난독화)
    pub buyer_uid: OpaqueId,                    // DB: buyer_id (난독화)
    pub seller_uid: OpaqueId,                   // DB: seller_id (난독화)
    pub currency: Currency,                     // DB: currency
    pub locked_funds: String,                   // DB: amount (String for U256 support)
    pub step: TradeStep,                        // DB: status
    pub auto_finalize_deadline: Option<String>, // DB: auto_confirm_at
    pub created_at: String,
}
