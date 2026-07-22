// crates/backend/src/infra/redis_pubsub_adapter.rs
// 목적: PubSubPort 구현체 — Redis connection manager를 통한 Pub/Sub 발행.

use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands};
use crate::ports::pubsub_port::{PubSubError, PubSubPort};

pub struct RedisPubSubAdapter {
    conn: ConnectionManager,
}

impl RedisPubSubAdapter {
    pub fn new(conn: ConnectionManager) -> Self {
        RedisPubSubAdapter { conn }
    }
}

#[async_trait]
impl PubSubPort for RedisPubSubAdapter {
    async fn publish(&self, channel: &str, message: &str) -> Result<(), PubSubError> {
        todo!("self.conn.publish(channel, message).await → PubSubError 매핑")
    }
}
