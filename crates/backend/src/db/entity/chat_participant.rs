// crates/backend/src/db/entity/chat_participant.rs
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "chat_participants")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub room_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i64,
    pub joined_at: Option<DateTimeWithTimeZone>,
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
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::chat_room::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChatRoom.def()
    }
}
impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}
