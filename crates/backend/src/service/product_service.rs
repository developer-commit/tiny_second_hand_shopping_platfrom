// crates/backend/src/service/product_service.rs
// 목적: 상품 등록/수정/삭제/검색 비즈니스 로직.

use sea_orm::DatabaseConnection;
use shared::dto::product_dto::{
    CreateItemReq, ItemDetailRes, ItemSummaryRes, ProductSearchQuery, UpdateItemReq, UpdateItemStateReq,
};
use thiserror::Error;

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

    /// 상품 목록 조회 및 검색 (GET /products)
    pub async fn list_products(
        &self,
        query: ProductSearchQuery,
    ) -> Result<Vec<ItemSummaryRes>, ProductServiceError> {
        todo!("products + product_tags LIKE 검색 → 이미지 썸네일 조인 → DTO 변환")
    }

    /// 상품 상세 조회 및 조회수 증가 (GET /products/{item_uid})
    pub async fn get_product(
        &self,
        product_id: i64,
    ) -> Result<ItemDetailRes, ProductServiceError> {
        todo!("products SELECT + view_count++ + images 조인 + tags 조인 → into_dto")
    }

    /// 상품 등록 (POST /products)
    pub async fn create_product(
        &self,
        seller_id: i64,
        req: CreateItemReq,
    ) -> Result<ItemDetailRes, ProductServiceError> {
        todo!("products INSERT → product_images INSERT → product_tags INSERT → into_dto")
    }

    /// 상품 정보 수정 (PUT /products/{item_uid})
    pub async fn update_product(
        &self,
        seller_id: i64,
        product_id: i64,
        req: UpdateItemReq,
    ) -> Result<ItemDetailRes, ProductServiceError> {
        todo!("소유자 검증 → products UPDATE → into_dto")
    }

    /// 상품 상태 변경 (PATCH /products/{item_uid}/status)
    pub async fn update_product_state(
        &self,
        seller_id: i64,
        product_id: i64,
        req: UpdateItemStateReq,
    ) -> Result<(), ProductServiceError> {
        todo!("소유자 검증 → products UPDATE status")
    }

    /// 상품 삭제 (DELETE /products/{item_uid})
    pub async fn delete_product(
        &self,
        seller_id: i64,
        product_id: i64,
    ) -> Result<(), ProductServiceError> {
        todo!("소유자 검증 → products DELETE (CASCADE로 images, tags 자동 삭제)")
    }
}
