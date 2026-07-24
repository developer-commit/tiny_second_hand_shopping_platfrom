// crates/backend/src/service/notification_service.rs
// 목적: 알림 DB 저장 및 Redis 발행 복합 서비스.

use crate::ports::{
    notification_port::{CreateNotificationCmd, NotificationPort, NotificationPortError},
    pubsub_port::PubSubPort,
};
use crate::service::traits::NotificationServiceTrait;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use shared::dto::noti_dto::NotificationRes;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotificationServiceError {
    #[error("알림을 찾을 수 없습니다.")]
    NotFound,
    #[error("내부 서버 오류: {0}")]
    Internal(String),
}

pub struct NotificationService {
    db: DatabaseConnection,
    pubsub: Arc<dyn PubSubPort>,
}

impl NotificationService {
    pub fn new(
        db: DatabaseConnection,
        pubsub: Arc<dyn crate::ports::pubsub_port::PubSubPort>,
    ) -> Self {
        NotificationService { db, pubsub }
    }
}

#[async_trait]
impl NotificationServiceTrait for NotificationService {
    async fn get_notifications(
        &self,
        user_id: i64,
    ) -> Result<Vec<NotificationRes>, NotificationServiceError> {
        use crate::db::entity::notification;
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

        let notifs = notification::Entity::find()
            .filter(notification::Column::UserId.eq(user_id))
            .order_by_desc(notification::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| NotificationServiceError::Internal(e.to_string()))?;

        let mut res = Vec::new();
        for n in notifs {
            if let Ok(dto) = n.try_into() {
                res.push(dto);
            }
        }

        Ok(res)
    }

    /// 알림 읽음 처리
    async fn mark_as_read(
        &self,
        user_id: i64,
        noti_id: i64,
    ) -> Result<(), NotificationServiceError> {
        use crate::db::entity::notification;
        use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

        let noti = notification::Entity::find()
            .filter(notification::Column::Id.eq(noti_id))
            .filter(notification::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|e| NotificationServiceError::Internal(e.to_string()))?
            .ok_or(NotificationServiceError::NotFound)?;

        let mut active_noti: notification::ActiveModel = noti.into();
        active_noti.is_read = Set(true);
        active_noti
            .update(&self.db)
            .await
            .map_err(|e| NotificationServiceError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn send_notification(
        &self,
        user_id: i64,
        noti_type: &str,
        reference_id: Option<i64>,
        message: &str,
    ) -> Result<(), NotificationServiceError> {
        let cmd = CreateNotificationCmd {
            user_id,
            noti_type: noti_type.to_string(),
            reference_id,
            message: message.to_string(),
        };
        self.send(cmd)
            .await
            .map_err(|e| NotificationServiceError::Internal(e.to_string()))
    }
}

#[async_trait::async_trait]
impl NotificationPort for NotificationService {
    /// Port 구현: 알림 DB 저장 + Redis Pub/Sub 발행
    async fn send(&self, cmd: CreateNotificationCmd) -> Result<(), NotificationPortError> {
        use crate::db::entity::notification;
        use sea_orm::{ActiveModelTrait, Set};

        let active_noti = notification::ActiveModel {
            user_id: Set(cmd.user_id),
            r#type: Set(cmd.noti_type.clone()),
            reference_id: Set(cmd.reference_id),
            message: Set(cmd.message.clone()),
            is_read: Set(false),
            created_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };

        let inserted = active_noti
            .insert(&self.db)
            .await
            .map_err(|e| NotificationPortError::SaveFailed(e.to_string()))?;

        let dto_result: Result<NotificationRes, _> = inserted.try_into();
        if let Ok(dto) = dto_result {
            if let Ok(payload) = serde_json::to_string(&dto) {
                let channel = format!("user:{}:notifications", cmd.user_id);
                if let Err(e) = self.pubsub.publish(&channel, &payload).await {
                    eprintln!("PubSub publish failed: {:?}", e);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::entity::notification;
    use chrono::Utc;
    use mockall::mock;
    use sea_orm::{DatabaseBackend, MockDatabase};

    mock! {
        pub PubSub {}
        #[async_trait::async_trait]
        impl PubSubPort for PubSub {
            async fn publish(&self, channel: &str, message: &str) -> Result<(), crate::ports::pubsub_port::PubSubError>;
        }
    }

    #[tokio::test]
    async fn test_get_notifications() {
        let now = Utc::now().into();
        let noti_model = notification::Model {
            id: 1,
            user_id: 100,
            r#type: "chat".to_string(),
            reference_id: Some(200),
            message: "hello".to_string(),
            is_read: false,
            created_at: now,
        };

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![noti_model]])
            .into_connection();

        let pubsub = Arc::new(MockPubSub::new());
        let service = NotificationService::new(db, pubsub);

        let res = service.get_notifications(100).await;
        assert!(res.is_ok());
        let notifs = res.unwrap();
        assert_eq!(notifs.len(), 1);
        assert_eq!(notifs[0].content, "hello");
    }

    #[tokio::test]
    async fn test_mark_as_read() {
        let now = Utc::now().into();
        let noti_model = notification::Model {
            id: 1,
            user_id: 100,
            r#type: "chat".to_string(),
            reference_id: Some(200),
            message: "hello".to_string(),
            is_read: false,
            created_at: now,
        };

        let updated_model = notification::Model {
            is_read: true,
            ..noti_model.clone()
        };

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![noti_model]])
            .append_query_results([vec![updated_model]])
            .into_connection();

        let pubsub = Arc::new(MockPubSub::new());
        let service = NotificationService::new(db, pubsub);

        let res = service.mark_as_read(100, 1).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_send_notification() {
        let now = Utc::now().into();
        let noti_model = notification::Model {
            id: 1,
            user_id: 100,
            r#type: "system".to_string(),
            reference_id: Some(0),
            message: "alert".to_string(),
            is_read: false,
            created_at: now,
        };

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![noti_model]])
            .into_connection();

        let mut mock_pubsub = MockPubSub::new();
        mock_pubsub
            .expect_publish()
            .with(
                mockall::predicate::eq("user:100:notifications"),
                mockall::predicate::always(),
            )
            .times(1)
            .returning(|_, _| Ok(()));

        let pubsub = Arc::new(mock_pubsub);
        let service = NotificationService::new(db, pubsub);

        let cmd = CreateNotificationCmd {
            user_id: 100,
            noti_type: "system".to_string(),
            reference_id: Some(0),
            message: "alert".to_string(),
        };

        let res = service.send(cmd).await;
        assert!(res.is_ok());
    }
}
