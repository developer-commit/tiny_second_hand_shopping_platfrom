// crates/backend/src/db/entity/notification.rs
use sea_orm::entity::prelude::*;
use shared::dto::noti_dto::{NotificationKind, NotificationRes};
use crate::utils::security::{obfuscate, SecurityError};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "notifications")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub r#type: String,                // "chat" | "escrow_update" | "system" | "warning"
    pub reference_id: Option<i64>,     // 관련 엔티티 내부 ID
    pub message: String,
    pub is_read: bool,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::user::Entity", from = "Column::UserId", to = "super::user::Column::Id")]
    User,
}

impl ActiveModelBehavior for ActiveModel {}

impl TryFrom<Model> for NotificationRes {
    type Error = SecurityError;

    fn try_from(m: Model) -> Result<Self, Self::Error> {
        let kind = match m.r#type.as_str() {
            "chat"          => NotificationKind::Chat,
            "escrow_update" => NotificationKind::EscrowUpdate,
            "system"        => NotificationKind::System,
            "warning"       => NotificationKind::Warning,
            _               => NotificationKind::System,
        };
        let link_uid = m.reference_id
            .map(|id| obfuscate(id))
            .transpose()?;
        Ok(NotificationRes {
            noti_uid: obfuscate(m.id)?,
            kind,
            link_uid,
            content: m.message,
            is_read: m.is_read,
            received_at: m.created_at.to_rfc3339(),
        })
    }
}
