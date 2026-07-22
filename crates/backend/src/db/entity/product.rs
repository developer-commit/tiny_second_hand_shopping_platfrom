// crates/backend/src/db/entity/product.rs
// 목적: products 테이블 Entity.
// TryFrom<Model> for ItemDetailRes는 이미지/태그 관계 데이터 없이는 완성 불가 →
// Service 계층에서 관계 데이터를 별도 조회 후 into_dto()로 조합합니다.

use sea_orm::entity::prelude::*;
use shared::dto::product_dto::{ItemDetailRes, ItemState};
use crate::utils::security::{obfuscate, SecurityError};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "products")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub seller_id: i64,
    pub title: String,
    pub description: String,
    pub price: Decimal,
    pub category: String,
    pub status: String,              // "on_sale" | "reserved" | "sold"
    pub view_count: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::user::Entity", from = "Column::SellerId", to = "super::user::Column::Id")]
    Seller,
    #[sea_orm(has_many = "super::product_image::Entity")]
    Images,
    #[sea_orm(has_many = "super::product_tag::Entity")]
    Tags,
    #[sea_orm(has_many = "super::escrow_trade::Entity")]
    EscrowTrades,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef { Relation::Seller.def() }
}

impl Related<super::product_image::Entity> for Entity {
    fn to() -> RelationDef { Relation::Images.def() }
}

impl Related<super::product_tag::Entity> for Entity {
    fn to() -> RelationDef { Relation::Tags.def() }
}

impl Related<super::escrow_trade::Entity> for Entity {
    fn to() -> RelationDef { Relation::EscrowTrades.def() }
}

impl Model {
    /// 관계 데이터(이미지 URL, 태그)를 주입받아 DTO로 변환.
    /// Service 계층에서 관계 데이터를 미리 조회 후 호출합니다.
    pub fn into_dto(
        self,
        images: Vec<String>,
        tags: Vec<String>,
    ) -> Result<ItemDetailRes, SecurityError> {
        let current_state = match self.status.as_str() {
            "on_sale"  => ItemState::OnSale,
            "reserved" => ItemState::Reserved,
            "sold"     => ItemState::Sold,
            _          => ItemState::OnSale,
        };
        Ok(ItemDetailRes {
            item_uid: obfuscate(self.id)?,
            owner_uid: obfuscate(self.seller_id)?,
            heading: self.title,
            detail_body: self.description,
            asking_price: self.price.try_into().unwrap_or(0.0),
            group_category: self.category,
            current_state,
            hit_count: self.view_count,
            image_urls: images,
            tags,
            listed_at: self.created_at.to_rfc3339(),
        })
    }
}
