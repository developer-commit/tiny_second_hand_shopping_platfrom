// crates/backend/src/ports/pubsub_port.rs
// 목적: Redis Pub/Sub 기반 실시간 메시징 Port.
// 채팅 메시지 및 알림을 Redis 채널로 발행합니다.
//
// [설계 주의]
// dyn 호환성을 위해 trait에는 async fn publish()만 포함.
// 채널명 생성 헬퍼는 자유 함수로 분리합니다.

use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PubSubError {
    #[error("발행 실패: {0}")]
    PublishFailed(String),
    #[error("구독 실패: {0}")]
    SubscribeFailed(String),
    #[error("연결 끊김")]
    Disconnected,
}

/// Redis Pub/Sub Port
///
/// 구현체: infra::redis_pubsub_adapter::RedisPubSubAdapter
#[async_trait]
pub trait PubSubPort: Send + Sync {
    /// 특정 채널에 메시지 발행
    async fn publish(&self, channel: &str, message: &str) -> Result<(), PubSubError>;
}

// ─── 채널명 생성 헬퍼 (자유 함수) ────────────────────────────────────────────
// static fn을 trait에 포함하면 dyn 호환성이 깨지므로 자유 함수로 분리합니다.

/// 채팅 채널명 생성: "chat:room:{room_id}"
pub fn chat_channel(room_id: i64) -> String {
    format!("chat:room:{room_id}")
}

/// 알림 채널명 생성: "noti:user:{user_id}"
pub fn notification_channel(user_id: i64) -> String {
    format!("noti:user:{user_id}")
}
