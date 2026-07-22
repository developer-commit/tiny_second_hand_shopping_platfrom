// crates/backend/src/service/user_service.rs
// 목적: 사용자 프로필 조회 및 수정 비즈니스 로직.

use sea_orm::DatabaseConnection;
use shared::dto::user_dto::{UpdateProfileReq, UserProfileRes};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserServiceError {
    #[error("사용자를 찾을 수 없습니다.")]
    NotFound,
    #[error("내부 서버 오류: {0}")]
    Internal(String),
}

pub struct UserService {
    db: DatabaseConnection,
}

impl UserService {
    pub fn new(db: DatabaseConnection) -> Self {
        UserService { db }
    }

    /// 내 프로필 조회 (GET /users/me)
    pub async fn get_profile(&self, user_id: i64) -> Result<UserProfileRes, UserServiceError> {
        todo!("users 테이블 SELECT WHERE id = user_id → TryFrom<Model> for UserProfileRes")
    }

    /// 프로필 수정 (PATCH /users/me)
    pub async fn update_profile(
        &self,
        user_id: i64,
        req: UpdateProfileReq,
    ) -> Result<UserProfileRes, UserServiceError> {
        todo!("users 테이블 UPDATE bio WHERE id = user_id")
    }

    /// 신뢰도 점수 재계산 — 리뷰 완료 후 escrow_service에서 호출
    pub async fn recalculate_trust_score(
        &self,
        user_id: i64,
    ) -> Result<(), UserServiceError> {
        todo!("reviews 테이블 AVG(rating) WHERE reviewee_id = user_id → users UPDATE trust_score")
    }
}
