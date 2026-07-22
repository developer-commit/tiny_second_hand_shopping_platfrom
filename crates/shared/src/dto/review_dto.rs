// crates/shared/src/dto/review_dto.rs
// 목적: 거래 완료 후 리뷰 및 평점 작성 관련 DTO.
// openapi: score(rating), feedback(comment), trade_uid(trade_id 난독화)

use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::types::OpaqueId;

/// [Request] POST /escrow/{trade_uid}/reviews — 리뷰 및 평점 작성
/// score는 1~5 범위로 validator가 컴파일 타임에 강제합니다.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SubmitReviewReq {
    #[validate(range(min = 1, max = 5))]
    pub score: i32,                  // DB: rating (1~5점)
    #[validate(length(max = 500))]
    pub feedback: Option<String>,    // DB: comment
}

/// [Response] 리뷰 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRes {
    pub review_uid: OpaqueId,        // DB: id (난독화)
    pub trade_uid: OpaqueId,         // DB: trade_id (난독화)
    pub reviewer_uid: OpaqueId,      // DB: reviewer_id (난독화)
    pub reviewee_uid: OpaqueId,      // DB: reviewee_id (난독화)
    pub score: i32,                  // DB: rating
    pub feedback: Option<String>,    // DB: comment
    pub written_at: String,          // DB: created_at
}
