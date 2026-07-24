// crates/shared/src/dto/product_dto.rs
// 목적: 상품 등록/수정/조회 관련 DTO.
// 스키마 은닉:
// - title → heading, description → detail_body
// - price → asking_price, category → group_category
// - status → current_state, view_count → hit_count
// - seller_id → owner_uid (난독화)

use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::types::OpaqueId;
use super::common_dto::Currency;

/// 상품 상태를 타입 안전하게 표현하는 Enum.
/// String 대신 Enum을 사용하여 유효하지 않은 상태값 전달을 컴파일 타임에 차단합니다.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ItemState {
    OnSale,    // DB: "on_sale"
    Reserved,  // DB: "reserved"
    Sold,      // DB: "sold"
}

/// [Request] POST /products — 상품 등록
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateItemReq {
    #[validate(length(min = 2, max = 100))]
    pub heading: String,             // DB: title
    #[validate(length(min = 1, max = 5000))]
    pub detail_body: String,         // DB: description
    #[validate(range(min = 0.000_01))]
    pub asking_price: f64,           // DB: price (For ETH, may want string, but let's keep f64 for now or let frontend handle it)
    pub currency: Currency,          // DB: currency
    #[validate(length(min = 1, max = 50))]
    pub group_category: String,      // DB: category
    pub item_tags: Vec<String>,      // DB: product_tags 테이블
    pub image_urls: Vec<String>,     // DB: product_images 테이블
}

/// [Request] PATCH /products/{item_uid}/status — 상품 상태 변경
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateItemStateReq {
    pub current_state: ItemState,    // DB: status
}

/// [Request] PUT /products/{item_uid} — 상품 정보 수정
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateItemReq {
    #[validate(length(min = 2, max = 100))]
    pub heading: Option<String>,
    #[validate(length(min = 1, max = 5000))]
    pub detail_body: Option<String>,
    #[validate(range(min = 0.000_01))]
    pub asking_price: Option<f64>,
    pub group_category: Option<String>,
    pub item_tags: Option<Vec<String>>,
}

/// [Response] 상품 상세 / 목록 공통 응답
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDetailRes {
    pub item_uid: OpaqueId,          // DB: id (난독화)
    pub owner_uid: OpaqueId,         // DB: seller_id (난독화)
    pub seller_trust_score: Option<f64>, // DB: users.trust_score
    pub heading: String,             // DB: title
    pub detail_body: String,         // DB: description
    pub asking_price: f64,           // DB: price
    pub currency: Currency,          // DB: currency
    pub group_category: String,      // DB: category
    pub current_state: ItemState,    // DB: status
    pub hit_count: i32,              // DB: view_count
    pub image_urls: Vec<String>,     // product_images 조인
    pub tags: Vec<String>,           // product_tags 조인
    pub listed_at: String,           // DB: created_at
}

/// [Response] 상품 목록 (썸네일용 축약 버전)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSummaryRes {
    pub item_uid: OpaqueId,
    pub heading: String,
    pub asking_price: f64,
    pub thumbnail_url: Option<String>,
    pub currency: Currency,
    pub current_state: ItemState,
    pub hit_count: i32,
    pub listed_at: String,
}

/// [Query] GET /products — 상품 목록 검색 쿼리 파라미터
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductSearchQuery {
    pub keyword: Option<String>,
    pub group_category: Option<String>,
    pub item_tag: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}
