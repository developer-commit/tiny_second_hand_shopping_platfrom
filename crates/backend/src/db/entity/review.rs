// crates/backend/src/db/entity/review.rs
use crate::utils::security::{SecurityError, obfuscate};
use sea_orm::entity::prelude::*;
use shared::dto::review_dto::ReviewRes;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "reviews")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub trade_id: i64,
    pub reviewer_id: i64,
    pub reviewee_id: i64,
    pub rating: i32,
    pub comment: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::escrow_trade::Entity",
        from = "Column::TradeId",
        to = "super::escrow_trade::Column::Id"
    )]
    EscrowTrade,
}

impl ActiveModelBehavior for ActiveModel {}

impl TryFrom<Model> for ReviewRes {
    type Error = SecurityError;

    fn try_from(m: Model) -> Result<Self, Self::Error> {
        Ok(ReviewRes {
            review_uid: obfuscate(m.id)?,
            trade_uid: obfuscate(m.trade_id)?,
            reviewer_uid: obfuscate(m.reviewer_id)?,
            reviewee_uid: obfuscate(m.reviewee_id)?,
            score: m.rating,
            feedback: m.comment,
            written_at: m.created_at.to_rfc3339(),
        })
    }
}
