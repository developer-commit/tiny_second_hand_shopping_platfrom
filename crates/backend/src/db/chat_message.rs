use sea_orm::entity::prelude::*;
use crate::utils::security::obfuscate_id;

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
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn into_dto(self) -> shared::dto::chat_dto::ChatMessagePayload {
        shared::dto::chat_dto::ChatMessagePayload {
            msg_uid: obfuscate_id(self.id),
            room_id: obfuscate_id(self.room_id),
            sender_uid: obfuscate_id(self.sender_id),
            text: self.message,
            read_status: self.is_read,
        }
    }
}
