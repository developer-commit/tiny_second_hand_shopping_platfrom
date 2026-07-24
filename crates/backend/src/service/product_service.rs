// crates/backend/src/service/product_service.rs
// 목적: 상품 등록/수정/삭제/검색 비즈니스 로직.

use sea_orm::DatabaseConnection;
use shared::dto::product_dto::{
    CreateItemReq, ItemDetailRes, ItemSummaryRes, ProductSearchQuery, UpdateItemReq, UpdateItemStateReq,
};
use thiserror::Error;
use async_trait::async_trait;
use crate::service::traits::ProductServiceTrait;
use rust_decimal::prelude::ToPrimitive;

#[derive(Debug, Error)]
pub enum ProductServiceError {
    #[error("상품을 찾을 수 없습니다.")]
    NotFound,
    #[error("권한이 없습니다 — 상품 소유자만 수정 가능")]
    Forbidden,
    #[error("내부 서버 오류: {0}")]
    Internal(String),
}

pub struct ProductService {
    db: DatabaseConnection,
}

impl ProductService {
    pub fn new(db: DatabaseConnection) -> Self {
        ProductService { db }
    }
}

#[async_trait]
impl ProductServiceTrait for ProductService {
    /// 상품 목록 조회 및 검색 (GET /products)
    async fn list_products(
        &self,
        query: ProductSearchQuery,
    ) -> Result<Vec<ItemSummaryRes>, ProductServiceError> {
        use crate::db::entity::product;
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, QuerySelect, QueryOrder};
        
        let mut filter = product::Entity::find();
        
        if let Some(k) = &query.keyword {
            filter = filter.filter(product::Column::Title.contains(k));
        }
        if let Some(c) = &query.group_category {
            filter = filter.filter(product::Column::Category.eq(c));
        }
        
        if let Some(t) = &query.item_tag {
            use sea_orm::RelationTrait;
            filter = filter.join(sea_orm::JoinType::InnerJoin, product::Relation::Tags.def());
            filter = filter.filter(crate::db::entity::product_tag::Column::TagName.eq(t));
        }
        
        let products = filter
            .order_by_desc(product::Column::CreatedAt)
            .limit(query.page_size.unwrap_or(20))
            .all(&self.db)
            .await
            .map_err(|e| ProductServiceError::Internal(e.to_string()))?;
            
        use crate::utils::security::obfuscate;
        use shared::dto::product_dto::ItemState;
        let mut result = Vec::new();
        for p in products {
            let current_state = match p.status.as_str() {
                "on_sale"  => ItemState::OnSale,
                "reserved" => ItemState::Reserved,
                "sold"     => ItemState::Sold,
                _          => ItemState::OnSale,
            };
            
            let currency_enum = match p.currency.as_str() {
                "ETH" => shared::dto::common_dto::Currency::ETH,
                _ => shared::dto::common_dto::Currency::UNSUPPORTED,
            };
            
            result.push(ItemSummaryRes {
                item_uid: obfuscate(p.id).unwrap_or_default(),
                heading: p.title,
                asking_price: p.price.try_into().unwrap_or(0.0),
                thumbnail_url: None, // Simplified
                currency: currency_enum,
                current_state,
                hit_count: p.view_count,
                listed_at: p.created_at.to_rfc3339(),
            });
        }
        Ok(result)
    }

    async fn get_product(
        &self,
        product_id: i64,
    ) -> Result<ItemDetailRes, ProductServiceError> {
        use crate::db::entity::{product, product_image, product_tag};
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set};
        
        let p = product::Entity::find_by_id(product_id)
            .one(&self.db)
            .await
            .map_err(|e| ProductServiceError::Internal(e.to_string()))?
            .ok_or(ProductServiceError::NotFound)?;
            
        let mut active_p: product::ActiveModel = p.clone().into();
        active_p.view_count = Set(p.view_count + 1);
        active_p.update(&self.db).await.map_err(|e| ProductServiceError::Internal(e.to_string()))?;
        
        let images = product_image::Entity::find()
            .filter(product_image::Column::ProductId.eq(product_id))
            .all(&self.db)
            .await
            .map_err(|e| ProductServiceError::Internal(e.to_string()))?
            .into_iter().map(|img| img.image_url).collect();
            
        let tags = product_tag::Entity::find()
            .filter(product_tag::Column::ProductId.eq(product_id))
            .all(&self.db)
            .await
            .map_err(|e| ProductServiceError::Internal(e.to_string()))?
            .into_iter().map(|t| t.tag_name).collect();
            
        let seller = crate::db::entity::user::Entity::find_by_id(p.seller_id)
            .one(&self.db)
            .await
            .map_err(|e| ProductServiceError::Internal(e.to_string()))?
            .ok_or(ProductServiceError::NotFound)?;
            
        p.into_dto(images, tags, Some(seller.trust_score.to_f64().unwrap_or(0.0))).map_err(|e| ProductServiceError::Internal(e.to_string()))
    }

    async fn create_product(
        &self,
        seller_id: i64,
        req: CreateItemReq,
    ) -> Result<ItemDetailRes, ProductServiceError> {
        use crate::db::entity::{product, product_image, product_tag};
        use sea_orm::{EntityTrait, ActiveModelTrait, Set, TransactionTrait};
        use rust_decimal::Decimal;
        use rust_decimal::prelude::FromPrimitive;
        
        let txn = self.db.begin().await.map_err(|e| ProductServiceError::Internal(e.to_string()))?;
        
        let price = Decimal::from_f64(req.asking_price).unwrap_or(Decimal::new(0, 0));
        
        let new_product = product::ActiveModel {
            seller_id: Set(seller_id),
            title: Set(req.heading.clone()),
            description: Set(req.detail_body.clone()),
            price: Set(price),
            category: Set(req.group_category.clone()),
            status: Set("on_sale".to_string()),
            view_count: Set(0),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };
        
        let inserted_p = new_product.insert(&txn).await.map_err(|e| ProductServiceError::Internal(e.to_string()))?;
        
        for url in req.image_urls.clone() {
            let img = product_image::ActiveModel {
                product_id: Set(inserted_p.id),
                image_url: Set(url),
                created_at: Set(chrono::Utc::now().into()),
                ..Default::default()
            };
            img.insert(&txn).await.map_err(|e| ProductServiceError::Internal(e.to_string()))?;
        }
        
        for tag in req.item_tags.clone() {
            let tg = product_tag::ActiveModel {
                product_id: Set(inserted_p.id),
                tag_name: Set(tag),
                ..Default::default()
            };
            tg.insert(&txn).await.map_err(|e| ProductServiceError::Internal(e.to_string()))?;
        }
        
        txn.commit().await.map_err(|e| ProductServiceError::Internal(e.to_string()))?;
        
        let seller = crate::db::entity::user::Entity::find_by_id(seller_id)
            .one(&self.db)
            .await
            .map_err(|e| ProductServiceError::Internal(e.to_string()))?
            .ok_or(ProductServiceError::NotFound)?;
            
        inserted_p.into_dto(req.image_urls, req.item_tags, Some(seller.trust_score.to_f64().unwrap_or(0.0))).map_err(|e| ProductServiceError::Internal(e.to_string()))
    }

    async fn update_product(
        &self,
        seller_id: i64,
        product_id: i64,
        req: UpdateItemReq,
    ) -> Result<ItemDetailRes, ProductServiceError> {
        use crate::db::entity::{product, product_image, product_tag};
        use sea_orm::{EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait};
        use rust_decimal::Decimal;
        use rust_decimal::prelude::FromPrimitive;
        
        let p = product::Entity::find_by_id(product_id)
            .one(&self.db)
            .await
            .map_err(|e| ProductServiceError::Internal(e.to_string()))?
            .ok_or(ProductServiceError::NotFound)?;
            
        if p.seller_id != seller_id {
            return Err(ProductServiceError::Forbidden);
        }
        
        let mut active_p: product::ActiveModel = p.clone().into();
        
        if let Some(h) = req.heading { active_p.title = Set(h); }
        if let Some(d) = req.detail_body { active_p.description = Set(d); }
        if let Some(pr) = req.asking_price {
            let dec = Decimal::from_f64(pr).unwrap_or(Decimal::new(0, 0));
            active_p.price = Set(dec);
        }
        if let Some(c) = req.group_category { active_p.category = Set(c); }
        active_p.updated_at = Set(chrono::Utc::now().into());
        
        let updated_p = active_p.update(&self.db).await.map_err(|e| ProductServiceError::Internal(e.to_string()))?;
        
        let images = product_image::Entity::find()
            .filter(product_image::Column::ProductId.eq(product_id))
            .all(&self.db)
            .await
            .map_err(|e| ProductServiceError::Internal(e.to_string()))?
            .into_iter().map(|img| img.image_url).collect();
            
        let tags = product_tag::Entity::find()
            .filter(product_tag::Column::ProductId.eq(product_id))
            .all(&self.db)
            .await
            .map_err(|e| ProductServiceError::Internal(e.to_string()))?
            .into_iter().map(|t| t.tag_name).collect();
            
        let seller = crate::db::entity::user::Entity::find_by_id(seller_id)
            .one(&self.db)
            .await
            .map_err(|e| ProductServiceError::Internal(e.to_string()))?
            .ok_or(ProductServiceError::NotFound)?;
            
        updated_p.into_dto(images, tags, Some(seller.trust_score.to_f64().unwrap_or(0.0))).map_err(|e| ProductServiceError::Internal(e.to_string()))
    }

    async fn update_product_state(
        &self,
        seller_id: i64,
        product_id: i64,
        req: UpdateItemStateReq,
    ) -> Result<(), ProductServiceError> {
        use crate::db::entity::product;
        use sea_orm::{EntityTrait, ActiveModelTrait, Set};
        use shared::dto::product_dto::ItemState;
        
        let p = product::Entity::find_by_id(product_id)
            .one(&self.db)
            .await
            .map_err(|e| ProductServiceError::Internal(e.to_string()))?
            .ok_or(ProductServiceError::NotFound)?;
            
        if p.seller_id != seller_id {
            return Err(ProductServiceError::Forbidden);
        }
        
        let status_str = match req.current_state {
            ItemState::OnSale => "on_sale",
            ItemState::Reserved => "reserved",
            ItemState::Sold => "sold",
        };
        
        let mut active_p: product::ActiveModel = p.into();
        active_p.status = Set(status_str.to_string());
        active_p.updated_at = Set(chrono::Utc::now().into());
        
        active_p.update(&self.db).await.map_err(|e| ProductServiceError::Internal(e.to_string()))?;
        
        Ok(())
    }

    async fn delete_product(
        &self,
        seller_id: i64,
        product_id: i64,
    ) -> Result<(), ProductServiceError> {
        use crate::db::entity::product;
        use sea_orm::{EntityTrait, ModelTrait};
        
        let p = product::Entity::find_by_id(product_id)
            .one(&self.db)
            .await
            .map_err(|e| ProductServiceError::Internal(e.to_string()))?
            .ok_or(ProductServiceError::NotFound)?;
            
        if p.seller_id != seller_id {
            return Err(ProductServiceError::Forbidden);
        }
        
        p.delete(&self.db).await.map_err(|e| ProductServiceError::Internal(e.to_string()))?;
        
        Ok(())
    }
}
