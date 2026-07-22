// crates/backend/src/handlers/product_handler.rs
// 목적: 상품 CRUD 및 검색 HTTP 핸들러.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use shared::dto::product_dto::{
    CreateItemReq, ItemDetailRes, ItemSummaryRes, ProductSearchQuery, UpdateItemReq, UpdateItemStateReq,
};
use crate::state::AppState;
use crate::utils::auth::Claims;

/// GET /v1/products (인증 불필요)
pub async fn list_products(
    State(state): State<AppState>,
    Query(query): Query<ProductSearchQuery>,
) -> Result<Json<Vec<ItemSummaryRes>>, StatusCode> {
    todo!("state.product_service.list_products(query).await → Json")
}

/// GET /v1/products/:item_uid (인증 불필요)
pub async fn get_product(
    State(state): State<AppState>,
    Path(item_uid): Path<String>,
) -> Result<Json<ItemDetailRes>, StatusCode> {
    todo!("deobfuscate(item_uid) → product_id → state.product_service.get_product(product_id).await → Json")
}

/// POST /v1/products (인증 필요)
pub async fn create_product(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<CreateItemReq>,
) -> Result<(StatusCode, Json<ItemDetailRes>), StatusCode> {
    todo!("validate → claims → seller_id → state.product_service.create_product(seller_id, req).await → 201")
}

/// PATCH /v1/products/:item_uid/status (인증 필요)
pub async fn update_product_status(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(item_uid): Path<String>,
    Json(req): Json<UpdateItemStateReq>,
) -> Result<StatusCode, StatusCode> {
    todo!("claims → seller_id, deobfuscate(item_uid) → state.product_service.update_product_state(seller_id, product_id, req).await")
}
