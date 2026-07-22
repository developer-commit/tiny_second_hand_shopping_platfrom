// crates/backend/src/ports/notification_port.rs
// 목적: 알림 생성 및 발송 Port.

use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotificationPortError {
    #[error("알림 저장 실패: {0}")]
    SaveFailed(String),
    #[error("알림 발송 실패: {0}")]
    DeliveryFailed(String),
}

/// 알림 생성 요청 구조체
#[derive(Debug, Clone)]
pub struct CreateNotificationCmd {
    pub user_id: i64,       // 수신자 내부 ID
    pub noti_type: String,  // "chat" | "escrow_update" | "system" | "warning"
    pub reference_id: Option<i64>,
    pub message: String,
}

/// 알림 Port
///
/// 구현체: NotificationService (DB 저장) + PubSubPort (실시간 발송)
#[async_trait]
pub trait NotificationPort: Send + Sync {
    /// 알림을 DB에 저장하고 Redis Pub/Sub으로 발송
    async fn send(
        &self,
        cmd: CreateNotificationCmd,
    ) -> Result<(), NotificationPortError>;
}
