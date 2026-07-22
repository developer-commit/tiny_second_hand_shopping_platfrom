// crates/backend/src/service/review_service.rs
// 목적: 거래 완료 후 리뷰 및 평점 작성 비즈니스 로직.

use std::sync::Arc;
use sea_orm::DatabaseConnection;
use shared::dto::review_dto::{ReviewRes, SubmitReviewReq};
use thiserror::Error;
use crate::service::user_service::UserService;

#[derive(Debug, Error)]
pub enum ReviewServiceError {
    #[error("거래를 찾을 수 없습니다.")]
    TradeNotFound,
    #[error("리뷰를 작성할 수 없는 거래 상태입니다.")]
    TradeNotCompleted,
    #[error("이미 리뷰를 작성했습니다.")]
    AlreadyReviewed,
    #[error("리뷰 작성 권한이 없습니다.")]
    Forbidden,
    #[error("내부 서버 오류: {0}")]
    Internal(String),
}

pub struct ReviewService {
    db: DatabaseConnection,
    user_service: Arc<UserService>,
}

impl ReviewService {
    pub fn new(db: DatabaseConnection, user_service: Arc<UserService>) -> Self {
        ReviewService { db, user_service }
    }

    /// 리뷰 작성 (POST /escrow/{trade_uid}/reviews)
    /// - 에스크로 status='settled'인 경우에만 허용 (1거래 1리뷰 UNIQUE 제약)
    /// - 리뷰 작성 후 상대방 신뢰도 점수 재계산
    pub async fn submit_review(
        &self,
        reviewer_id: i64,
        trade_id: i64,
        req: SubmitReviewReq,
    ) -> Result<ReviewRes, ReviewServiceError> {
        todo!("거래 조회 → settled 상태 확인 → 참가자 확인 → reviews INSERT → user_service.recalculate_trust_score")
    }
}
