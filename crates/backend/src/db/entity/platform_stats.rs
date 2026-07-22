// crates/backend/src/db/entity/platform_stats.rs
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "platform_stats")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub total_fee_collected: Decimal,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
