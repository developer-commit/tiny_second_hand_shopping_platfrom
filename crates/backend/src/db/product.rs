use sea_orm::entity::prelude::*;
use crate::utils::security::obfuscate_id;

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
    pub status: String,
    pub view_count: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// DB 모델을 네트워크 DTO로 변환 (관계 데이터 포함)
    pub fn into_dto(self, images: Vec<String>, tags: Vec<String>) -> shared::dto::product_dto::ItemDetailRes {
        shared::dto::product_dto::ItemDetailRes {
            item_uid: obfuscate_id(self.id),
            owner_uid: obfuscate_id(self.seller_id),
            heading: self.title,
            detail_body: self.description,
            asking_price: self.price.try_into().unwrap_or(0.0),
            current_state: self.status,
            hit_count: self.view_count,
            image_urls: images,
            tags,
        }
    }
}
