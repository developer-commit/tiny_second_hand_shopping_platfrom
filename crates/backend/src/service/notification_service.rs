// crates/backend/src/service/notification_service.rs
// 목적: 알림 DB 저장 및 Redis 발행 복합 서비스.

use std::sync::Arc;
use sea_orm::DatabaseConnection;
use shared::dto::noti_dto::NotificationRes;
use thiserror::Error;
use crate::ports::{
    notification_port::{CreateNotificationCmd, NotificationPort, NotificationPortError},
    pubsub_port::PubSubPort,
};

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
    pub fn new(db: DatabaseConnection, pubsub: Arc<dyn PubSubPort>) -> Self {
        NotificationService { db, pubsub }
    }

    /// 알림 목록 조회 (GET /notifications)
    pub async fn get_notifications(
        &self,
        user_id: i64,
    ) -> Result<Vec<NotificationRes>, NotificationServiceError> {
        todo!("notifications SELECT WHERE user_id = user_id ORDER BY created_at DESC → TryFrom<Model>")
    }

    /// 알림 읽음 처리
    pub async fn mark_as_read(
        &self,
        user_id: i64,
        noti_id: i64,
    ) -> Result<(), NotificationServiceError> {
        todo!("notifications UPDATE is_read=true WHERE id=noti_id AND user_id=user_id")
    }
}

#[async_trait::async_trait]
impl NotificationPort for NotificationService {
    /// Port 구현: 알림 DB 저장 + Redis Pub/Sub 발행
    async fn send(&self, cmd: CreateNotificationCmd) -> Result<(), NotificationPortError> {
        todo!("notifications INSERT → serde_json::to_string(payload) → pubsub.publish(notification_channel, payload)")
    }
}
