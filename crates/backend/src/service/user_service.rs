// crates/backend/src/service/user_service.rs
// 목적: 사용자 프로필 조회 및 수정 비즈니스 로직.

use crate::service::traits::UserServiceTrait;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use shared::dto::user_dto::{PublicUserProfileRes, UpdateProfileReq, UserProfileRes};
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
}

#[async_trait]
impl UserServiceTrait for UserService {
    /// 공개 프로필 조회
    async fn get_public_profile(
        &self,
        user_id: i64,
    ) -> Result<PublicUserProfileRes, UserServiceError> {
        use crate::db::entity::user;
        use crate::utils::security::obfuscate;
        use sea_orm::EntityTrait;

        let user_model = user::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| UserServiceError::Internal(e.to_string()))?
            .ok_or(UserServiceError::NotFound)?;

        use rust_decimal::prelude::ToPrimitive;
        Ok(PublicUserProfileRes {
            user_uid: obfuscate(user_model.id).unwrap_or_default(),
            display_name: user_model.username,
            bio: user_model.bio,
            reliability_index: user_model.trust_score.to_f64().unwrap_or(0.0),
            joined_at: user_model.created_at.to_rfc3339(),
        })
    }

    /// 내 프로필 조회
    async fn get_profile(&self, user_id: i64) -> Result<UserProfileRes, UserServiceError> {
        use crate::db::entity::user;
        use sea_orm::EntityTrait;

        let user_model = user::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| UserServiceError::Internal(e.to_string()))?
            .ok_or(UserServiceError::NotFound)?;

        user_model
            .try_into()
            .map_err(|e| UserServiceError::Internal(format!("Conversion error: {:?}", e)))
    }

    /// 프로필 수정 (PATCH /users/me)
    async fn update_profile(
        &self,
        user_id: i64,
        req: UpdateProfileReq,
    ) -> Result<UserProfileRes, UserServiceError> {
        use crate::db::entity::user;
        use sea_orm::{ActiveModelTrait, EntityTrait, Set};

        let user_model = user::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| UserServiceError::Internal(e.to_string()))?
            .ok_or(UserServiceError::NotFound)?;

        let mut active_user: user::ActiveModel = user_model.into();

        if let Some(bio) = req.bio {
            active_user.bio = Set(Some(bio));
        }
        active_user.updated_at = Set(chrono::Utc::now().into());

        let updated = active_user
            .update(&self.db)
            .await
            .map_err(|e| UserServiceError::Internal(e.to_string()))?;

        updated
            .try_into()
            .map_err(|e| UserServiceError::Internal(format!("Conversion error: {:?}", e)))
    }

    /// 신뢰도 점수 재계산
    async fn recalculate_trust_score(&self, user_id: i64) -> Result<(), UserServiceError> {
        use crate::db::entity::{review, user};
        use rust_decimal::Decimal;
        use rust_decimal::prelude::FromPrimitive;
        use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

        let reviews = review::Entity::find()
            .filter(review::Column::RevieweeId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(|e| UserServiceError::Internal(e.to_string()))?;

        let avg = if reviews.is_empty() {
            0.0
        } else {
            let sum: i32 = reviews.iter().map(|r| r.rating).sum();
            sum as f64 / reviews.len() as f64
        };

        let user_model = user::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| UserServiceError::Internal(e.to_string()))?
            .ok_or(UserServiceError::NotFound)?;

        let mut active_user: user::ActiveModel = user_model.into();
        active_user.trust_score = Set(Decimal::from_f64(avg).unwrap_or(Decimal::new(0, 0)));
        active_user
            .update(&self.db)
            .await
            .map_err(|e| UserServiceError::Internal(e.to_string()))?;

        Ok(())
    }
}
