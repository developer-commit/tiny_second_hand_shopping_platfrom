// crates/backend/src/service/review_service.rs
// 목적: 거래 완료 후 리뷰 및 평점 작성 비즈니스 로직.

use std::sync::Arc;
use sea_orm::DatabaseConnection;
use shared::dto::review_dto::{ReviewRes, SubmitReviewReq};
use crate::service::user_service::UserService;
use thiserror::Error;
use async_trait::async_trait;
use crate::service::traits::{ReviewServiceTrait, UserServiceTrait};

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
    user_service: Arc<dyn UserServiceTrait>,
}

impl ReviewService {
    pub fn new(db: DatabaseConnection, user_service: Arc<dyn UserServiceTrait>) -> Self {
        ReviewService { db, user_service }
    }
}

#[async_trait]
impl ReviewServiceTrait for ReviewService {
    /// 리뷰 작성 (POST /escrow/{trade_uid}/reviews)
    /// - 에스크로 status='settled'인 경우에만 허용 (1거래 1리뷰 UNIQUE 제약)
    /// - 리뷰 작성 후 상대방 신뢰도 점수 재계산
    async fn submit_review(
        &self,
        reviewer_id: i64,
        trade_id: i64,
        req: SubmitReviewReq,
    ) -> Result<ReviewRes, ReviewServiceError> {
        use crate::db::entity::{escrow_trade, review};
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set, TransactionTrait};
        use crate::utils::security::obfuscate;

        let txn = self.db.begin().await.map_err(|e| ReviewServiceError::Internal(e.to_string()))?;

        let trade = escrow_trade::Entity::find_by_id(trade_id)
            .one(&txn)
            .await
            .map_err(|e| ReviewServiceError::Internal(e.to_string()))?
            .ok_or(ReviewServiceError::TradeNotFound)?;

        if trade.status != "settled" {
            return Err(ReviewServiceError::TradeNotCompleted);
        }
        
        let review_target = if trade.buyer_id == reviewer_id {
            trade.seller_id
        } else if trade.seller_id == reviewer_id {
            trade.buyer_id
        } else {
            return Err(ReviewServiceError::Forbidden);
        };
        
        let existing_review = review::Entity::find()
            .filter(review::Column::TradeId.eq(trade_id))
            .filter(review::Column::ReviewerId.eq(reviewer_id))
            .one(&txn)
            .await
            .map_err(|e| ReviewServiceError::Internal(e.to_string()))?;
            
        if existing_review.is_some() {
            return Err(ReviewServiceError::AlreadyReviewed);
        }
        
        let r = review::ActiveModel {
            trade_id: Set(trade.id),
            reviewer_id: Set(reviewer_id),
            reviewee_id: Set(review_target),
            rating: Set(req.score),
            comment: Set(req.feedback),
            created_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };
        
        let inserted = r.insert(&txn).await.map_err(|e| ReviewServiceError::Internal(e.to_string()))?;
        
        txn.commit().await.map_err(|e| ReviewServiceError::Internal(e.to_string()))?;
        
        // Recalculate trust score asynchronously
        if let Err(e) = self.user_service.recalculate_trust_score(review_target).await {
            eprintln!("Failed to recalculate trust score: {}", e);
        }
        
        Ok(ReviewRes {
            review_uid: obfuscate(inserted.id).unwrap_or_default(),
            trade_uid: obfuscate(inserted.trade_id).unwrap_or_default(),
            reviewer_uid: obfuscate(inserted.reviewer_id).unwrap_or_default(),
            reviewee_uid: obfuscate(inserted.reviewee_id).unwrap_or_default(),
            score: inserted.rating,
            feedback: inserted.comment,
            written_at: inserted.created_at.to_rfc3339(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::entity::{escrow_trade, review, user};
    use sea_orm::{DatabaseBackend, MockDatabase};
    use chrono::Utc;
    use rust_decimal::Decimal;

    #[tokio::test]
    async fn test_submit_review_success() {
        let now = Utc::now().into();
        let trade_model = escrow_trade::Model {
            id: 1,
            product_id: 10,
            buyer_id: 100,
            seller_id: 200,
            currency: "BCH".to_string(),
            amount: Decimal::new(1000, 0),
            platform_fee: Decimal::new(10, 0),
            status: "settled".to_string(),
            auto_confirm_at: None,
            dispute_reason: None,
            blockchain_tx_hash: None,
            contract_trade_id: None,
            created_at: now,
            updated_at: now,
        };

        let review_model = review::Model {
            id: 1,
            trade_id: 1,
            reviewer_id: 100,
            reviewee_id: 200,
            rating: 5,
            comment: Some("Great".to_string()),
            created_at: now,
        };

        let user_model = user::Model {
            id: 200,
            username: "test".to_string(),
            password_hash: "hash".to_string(),
            email: Some("test@test.com".to_string()),
            phone: None,
            is_verified: true,
            bio: None,
            trust_score: Decimal::new(0, 0),
            is_2fa_enabled: false,
            two_factor_secret: None,
            role: "user".to_string(),
            status: "active".to_string(),
            reported_count: 0,
            created_at: now,
            updated_at: now,
        };

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![trade_model.clone()]])
            .append_query_results([vec![] as Vec<review::Model>])
            .append_query_results([vec![review_model.clone()]])
            // UserService uses the same DB connection, append results for recalculate_trust_score
            .append_query_results([vec![review_model.clone()]]) // reviews for reviewee
            .append_query_results([vec![user_model.clone()]])   // find user
            .append_query_results([vec![user_model.clone()]])   // update user return
            .into_connection();

        use crate::state::DbClone;
        let user_service = Arc::new(UserService::new(db.clone_conn()));
        let review_service = ReviewService::new(db, user_service);

        let req = SubmitReviewReq { score: 5, feedback: Some("Great".to_string()) };
        let res = review_service.submit_review(100, 1, req).await;
        
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.score, 5);
        assert_eq!(res.feedback.unwrap(), "Great");
    }

    #[tokio::test]
    async fn test_submit_review_trade_not_completed() {
        let now = Utc::now().into();
        let trade_model = escrow_trade::Model {
            id: 1,
            product_id: 10,
            buyer_id: 100,
            seller_id: 200,
            currency: "BCH".to_string(),
            amount: Decimal::new(1000, 0),
            platform_fee: Decimal::new(10, 0),
            status: "deposited".to_string(),
            auto_confirm_at: None,
            dispute_reason: None,
            blockchain_tx_hash: None,
            contract_trade_id: None,
            created_at: now,
            updated_at: now,
        };

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![trade_model]])
            .into_connection();

        use crate::state::DbClone;
        let user_service = Arc::new(UserService::new(db.clone_conn()));
        let review_service = ReviewService::new(db, user_service);

        let req = SubmitReviewReq { score: 5, feedback: Some("Great".to_string()) };
        let res = review_service.submit_review(100, 1, req).await;
        
        assert!(matches!(res, Err(ReviewServiceError::TradeNotCompleted)));
    }
}
