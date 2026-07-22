// crates/backend/src/db/entity/product_tag.rs
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "product_tags")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub product_id: i64,
    pub tag_name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::product::Entity", from = "Column::ProductId", to = "super::product::Column::Id")]
    Product,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::product::Entity> for Entity {
    fn to() -> RelationDef { Relation::Product.def() }
}
