// crates/backend/src/service/chat_service.rs
// 목적: 채팅방 관리 및 메시지 저장/발행 비즈니스 로직.

use std::sync::Arc;
use sea_orm::DatabaseConnection;
use shared::dto::chat_dto::{ChatMessagePayload, ChatRoomRes, CreateChatRoomReq, SendMessageReq};
use thiserror::Error;
use crate::ports::pubsub_port::PubSubPort;

#[derive(Debug, Error)]
pub enum ChatServiceError {
    #[error("채팅방을 찾을 수 없습니다.")]
    RoomNotFound,
    #[error("채팅방 접근 권한이 없습니다.")]
    Forbidden,
    #[error("내부 서버 오류: {0}")]
    Internal(String),
}

pub struct ChatService {
    db: DatabaseConnection,
    pubsub: Arc<dyn PubSubPort>,
}

impl ChatService {
    pub fn new(db: DatabaseConnection, pubsub: Arc<dyn PubSubPort>) -> Self {
        ChatService { db, pubsub }
    }

    /// 1:1 채팅방 생성 또는 기존 채팅방 반환
    pub async fn get_or_create_room(
        &self,
        user_id: i64,
        req: CreateChatRoomReq,
    ) -> Result<ChatRoomRes, ChatServiceError> {
        todo!("기존 채팅방 조회 → 없으면 chat_rooms INSERT + chat_participants INSERT")
    }

    /// 메시지 저장 및 Redis 발행 (WebSocket 핸들러에서 호출)
    pub async fn send_message(
        &self,
        sender_id: i64,
        req: SendMessageReq,
    ) -> Result<ChatMessagePayload, ChatServiceError> {
        todo!("참가자 검증 → chat_messages INSERT → pubsub.publish(chat_channel, payload) → DTO 반환")
    }

    /// 채팅 이력 조회
    pub async fn get_history(
        &self,
        user_id: i64,
        room_id: i64,
        before_msg_id: Option<i64>,
    ) -> Result<Vec<ChatMessagePayload>, ChatServiceError> {
        todo!("참가자 검증 → chat_messages SELECT WHERE room_id ORDER BY created_at DESC LIMIT 50")
    }
}
