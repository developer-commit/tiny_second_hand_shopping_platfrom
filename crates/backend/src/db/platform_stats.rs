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

impl Model {
    pub fn into_admin_dto(self) -> shared::dto::admin_dto::PlatformYieldRes {
        shared::dto::admin_dto::PlatformYieldRes {
            total_accumulated_bch: self.total_fee_collected.try_into().unwrap_or(0.0),
            last_calculated_at: self.updated_at.to_rfc3339(),
        }
    }
}
