// crates/backend/src/db/entity/wallet.rs
// 목적: wallets 테이블의 SeaORM Entity.
//
// [보안 설계]
// - bch_address는 공개 가능한 입금 주소 (CashAddr 형식)
// - 지갑 개인키(private key)는 이 Entity에 절대 포함되지 않습니다.
//   개인키는 별도의 HSM/KMS 기반 보안 저장소에서 관리하며,
//   필요 시 infra 계층의 BchWalletAdapter를 통해서만 접근합니다.

use sea_orm::entity::prelude::*;
use shared::dto::transaction_dto::WalletStateRes;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "wallets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,                         // FK → users.id
    pub bch_address: String,                   // BCH CashAddr 공개 주소
    pub balance: Decimal,                      // BCH 잔액
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::user::Entity", from = "Column::UserId", to = "super::user::Column::Id")]
    User,
    #[sea_orm(has_many = "super::wallet_transaction::Entity")]
    WalletTransactions,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef { Relation::User.def() }
}

impl Related<super::wallet_transaction::Entity> for Entity {
    fn to() -> RelationDef { Relation::WalletTransactions.def() }
}

impl Model {
    /// 잠금 금액을 주입받아 WalletStateRes DTO로 변환.
    /// locked_in_escrow는 escrow_trades에서 별도 집계한 값입니다.
    pub fn into_dto(self, locked_in_escrow: f64) -> WalletStateRes {
        WalletStateRes {
            public_address: self.bch_address,
            available_balance: self.balance.try_into().unwrap_or(0.0),
            locked_in_escrow,
        }
    }
}
