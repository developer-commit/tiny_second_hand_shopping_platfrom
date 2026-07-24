// crates/backend/src/db/entity/wallet_transaction.rs
// 목적: wallet_transactions 테이블 Entity.
// 스키마 은닉: tx_type→movement_type, network_fee→fee_deducted 등은 DTO 변환에서 적용

use crate::utils::security::{SecurityError, obfuscate};
use sea_orm::entity::prelude::*;
use shared::dto::transaction_dto::{MovementType, TxHistoryItemRes, TxStatus};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "wallet_transactions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub wallet_id: i64,
    pub tx_type: String, // "deposit" | "withdrawal"
    pub amount: Decimal,
    pub network_fee: Decimal,
    pub tx_hash: Option<String>,
    pub status: String, // "pending" | "confirmed" | "failed"
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::wallet::Entity",
        from = "Column::WalletId",
        to = "super::wallet::Column::Id"
    )]
    Wallet,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::wallet::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Wallet.def()
    }
}

impl TryFrom<Model> for TxHistoryItemRes {
    type Error = SecurityError;

    fn try_from(m: Model) -> Result<Self, Self::Error> {
        let movement_type = match m.tx_type.as_str() {
            "deposit" => MovementType::Deposit,
            "withdrawal" => MovementType::Withdrawal,
            _ => MovementType::Deposit, // fallback
        };
        let process_status = match m.status.as_str() {
            "pending" => TxStatus::Pending,
            "confirmed" => TxStatus::Confirmed,
            "failed" => TxStatus::Failed,
            _ => TxStatus::Pending,
        };
        Ok(TxHistoryItemRes {
            tx_uid: obfuscate(m.id)?,
            movement_type,
            amount: m.amount.try_into().unwrap_or(0.0),
            fee_deducted: m.network_fee.try_into().unwrap_or(0.0),
            blockchain_hash: m.tx_hash,
            process_status,
            timestamp: m.created_at.to_rfc3339(),
        })
    }
}
