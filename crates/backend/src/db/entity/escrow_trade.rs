// crates/backend/src/db/entity/escrow_trade.rs
// 목적: escrow_trades 테이블 Entity.
// 스키마 은닉: amount→locked_funds, status→step, auto_confirm_at→auto_finalize_deadline

use sea_orm::entity::prelude::*;
use shared::dto::escrow_dto::{SafeTradeStatusRes, TradeStep};
use crate::utils::security::{obfuscate, SecurityError};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "escrow_trades")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub product_id: i64,
    pub buyer_id: i64,
    pub seller_id: i64,
    pub amount: Decimal,
    pub platform_fee: Decimal,          // 프론트에 노출 금지 (수수료 정보 은닉)
    pub status: String,
    pub auto_confirm_at: Option<DateTimeWithTimeZone>,
    pub dispute_reason: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::product::Entity", from = "Column::ProductId", to = "super::product::Column::Id")]
    Product,
    #[sea_orm(belongs_to = "super::user::Entity", from = "Column::BuyerId", to = "super::user::Column::Id")]
    Buyer,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::product::Entity> for Entity {
    fn to() -> RelationDef { Relation::Product.def() }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef { Relation::Buyer.def() }
}

impl TryFrom<Model> for SafeTradeStatusRes {
    type Error = SecurityError;

    fn try_from(m: Model) -> Result<Self, Self::Error> {
        let step = match m.status.as_str() {
            "deposited" => TradeStep::Deposited,
            "received"  => TradeStep::Received,
            "disputed"  => TradeStep::Disputed,
            "settled"   => TradeStep::Settled,
            "refunded"  => TradeStep::Refunded,
            _           => TradeStep::Deposited,
        };
        Ok(SafeTradeStatusRes {
            trade_uid: obfuscate(m.id)?,
            item_uid: obfuscate(m.product_id)?,
            buyer_uid: obfuscate(m.buyer_id)?,
            seller_uid: obfuscate(m.seller_id)?,
            locked_funds: m.amount.try_into().unwrap_or(0.0),
            step,
            auto_finalize_deadline: m.auto_confirm_at.map(|t| t.to_rfc3339()),
            created_at: m.created_at.to_rfc3339(),
        })
    }
}
