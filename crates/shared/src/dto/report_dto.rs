// crates/shared/src/dto/report_dto.rs
// 목적: 불량 상품/유저 신고 DTO.
// 스키마 은닉: reason → cause, product_id → target_item_uid (난독화)
// 보안 제약: 1유저 1상품 1회 신고는 DB UNIQUE 제약조건으로 강제,
//           서비스 계층에서 이미 신고했는지 사전 검증합니다.

use crate::types::OpaqueId;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// 신고 처리 상태 Enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReportStatus {
    Pending,     // DB: "pending"
    Reviewed,    // DB: "reviewed"
    ActionTaken, // DB: "action_taken"
}

/// [Request] POST /products/{item_uid}/reports — 상품 신고
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SubmitReportReq {
    #[validate(length(min = 5, max = 1000))]
    pub cause: String, // DB: reason
}

/// [Response] 신고 접수 확인
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportAckRes {
    pub report_uid: OpaqueId, // DB: id (난독화)
    pub status: ReportStatus, // DB: status
    pub submitted_at: String, // DB: created_at
}
