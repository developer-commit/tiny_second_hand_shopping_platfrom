// crates/backend/src/db/entity/chat_message.rs
// 목적: chat_messages 테이블 Entity.
// BUG FIX: 기존 db/chat_message.rs의 into_dto()에서 room_id 필드명 오류(room_uid여야 함) 수정

use crate::utils::security::{SecurityError, obfuscate};
use sea_orm::entity::prelude::*;
use shared::dto::chat_dto::ChatMessagePayload;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "chat_messages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub room_id: i64,
    pub sender_id: i64,
    pub message: String,
    pub is_read: bool,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::chat_room::Entity",
        from = "Column::RoomId",
        to = "super::chat_room::Column::Id"
    )]
    ChatRoom,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::SenderId",
        to = "super::user::Column::Id"
    )]
    Sender,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::chat_room::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChatRoom.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sender.def()
    }
}

impl TryFrom<Model> for ChatMessagePayload {
    type Error = SecurityError;

    fn try_from(m: Model) -> Result<Self, Self::Error> {
        Ok(ChatMessagePayload {
            msg_uid: obfuscate(m.id)?,
            room_uid: obfuscate(m.room_id)?, // ← BUG FIX: 기존 코드의 room_id → room_uid
            sender_uid: obfuscate(m.sender_id)?,
            text: m.message,
            read_status: m.is_read,
            sent_at: m.created_at.to_rfc3339(),
        })
    }
}
