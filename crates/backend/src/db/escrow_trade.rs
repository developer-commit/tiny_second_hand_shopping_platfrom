use sea_orm::entity::prelude::*;
use crate::utils::security::obfuscate_id;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "escrow_trades")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub product_id: i64,
    pub buyer_id: i64,
    pub seller_id: i64,
    pub amount: Decimal,
    pub platform_fee: Decimal, // 프론트에는 노출하지 않거나 합산해서 전달
    pub status: String,
    pub auto_confirm_at: Option<DateTimeWithTimeZone>,
    pub dispute_reason: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn into_dto(self) -> shared::dto::escrow_dto::SafeTradeStatusRes {
        shared::dto::escrow_dto::SafeTradeStatusRes {
            trade_uid: obfuscate_id(self.id),
            item_uid: obfuscate_id(self.product_id),
            locked_funds: self.amount.try_into().unwrap_or(0.0),
            step: self.status,
            auto_finalize_deadline: self.auto_confirm_at.map(|t| t.to_rfc3339()),
        }
    }
}
