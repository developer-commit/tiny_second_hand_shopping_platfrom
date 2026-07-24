// crates/shared/src/dto/chat_dto.rs
// 목적: 채팅방 및 메시지 관련 DTO.
// 스키마 은닉: DB의 room_id → room_uid, sender_id → sender_uid, is_read → read_status
// 보안: 모든 참가자 ID는 OpaqueId로 난독화

use crate::types::OpaqueId;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// 채팅방 유형 Enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoomKind {
    Public,  // DB: "public" — 전체 채팅
    Private, // DB: "private" — 1:1 채팅
}

/// [Request] POST /chat/rooms — 1:1 채팅방 생성
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatRoomReq {
    pub item_uid: OpaqueId,    // DB: product_id (거래 연동)
    pub partner_uid: OpaqueId, // 상대방 user_uid
}

/// [Response] 채팅방 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRoomRes {
    pub room_uid: OpaqueId,         // DB: id (난독화)
    pub room_kind: RoomKind,        // DB: room_type
    pub item_uid: Option<OpaqueId>, // DB: product_id (난독화)
    pub product_name: Option<String>,
    pub participant_uids: Vec<OpaqueId>, // chat_participants 조인
    pub created_at: String,
}

/// [WebSocket Payload] 채팅 메시지 — WS로 송수신
/// msg_uid: 서버 저장 후 발급된 OpaqueId
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessagePayload {
    pub msg_uid: OpaqueId,    // DB: id (난독화)
    pub room_uid: OpaqueId,   // DB: room_id (난독화) ← 기존 BUG 수정
    pub sender_uid: OpaqueId, // DB: sender_id (난독화)
    pub text: String,         // DB: message
    pub read_status: bool,    // DB: is_read
    pub sent_at: String,      // DB: created_at
}

/// [Request] WebSocket 메시지 전송 Payload
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SendMessageReq {
    pub room_uid: OpaqueId,
    #[validate(length(min = 1, max = 5000))]
    pub text: String,
}
