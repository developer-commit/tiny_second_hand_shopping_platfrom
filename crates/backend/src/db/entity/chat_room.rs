// crates/backend/src/db/entity/chat_room.rs
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "chat_rooms")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub product_id: Option<i64>,
    pub room_type: String, // "public" | "private"
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::chat_message::Entity")]
    Messages,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::chat_message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}
