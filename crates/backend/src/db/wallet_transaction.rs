use sea_orm::entity::prelude::*;
use crate::utils::security::obfuscate_id;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "wallet_transactions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub wallet_id: i64,
    pub tx_type: String,
    pub amount: Decimal,
    pub network_fee: Decimal,
    pub tx_hash: Option<String>,
    pub status: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn into_dto(self) -> shared::dto::transaction_dto::TxHistoryItemRes {
        shared::dto::transaction_dto::TxHistoryItemRes {
            tx_uid: obfuscate_id(self.id),
            movement_type: self.tx_type,
            amount: self.amount.try_into().unwrap_or(0.0),
            fee_deducted: self.network_fee.try_into().unwrap_or(0.0),
            blockchain_hash: self.tx_hash,
            process_status: self.status,
            timestamp: self.created_at.to_rfc3339(),
        }
    }
}
