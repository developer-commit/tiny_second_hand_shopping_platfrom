// crates/backend/src/handlers/product_handler.rs
// 목적: 상품 CRUD 및 검색 HTTP 핸들러.

use crate::state::AppState;
use crate::utils::auth::Claims;
use crate::utils::error::AppError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use shared::dto::product_dto::{
    CreateItemReq, ItemDetailRes, ItemSummaryRes, ProductSearchQuery, UpdateItemReq,
    UpdateItemStateReq,
};

use crate::utils::security::deobfuscate;

/// GET /v1/products (인증 불필요)
pub async fn list_products(
    State(state): State<AppState>,
    Query(query): Query<ProductSearchQuery>,
) -> Result<Json<Vec<ItemSummaryRes>>, AppError> {
    state
        .product_service
        .list_products(query)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("list_products error: {:?}", e);
            AppError::Internal
        })
}

/// GET /v1/products/:item_uid (인증 불필요)
pub async fn get_product(
    State(state): State<AppState>,
    Path(item_uid): Path<String>,
) -> Result<Json<ItemDetailRes>, AppError> {
    let product_id =
        deobfuscate(&item_uid).map_err(|_| AppError::NotFound("Not found".to_string()))?;
    state
        .product_service
        .get_product(product_id)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("get_product error: {:?}", e);
            AppError::NotFound("Not found".to_string())
        })
}

/// POST /v1/products (인증 필요)
pub async fn create_product(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<CreateItemReq>,
) -> Result<(StatusCode, Json<ItemDetailRes>), AppError> {
    use validator::Validate;
    req.validate()
        .map_err(|_| AppError::BadRequest("Invalid request".to_string()))?;

    let seller_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    state
        .product_service
        .create_product(seller_id, req)
        .await
        .map(|res| (StatusCode::CREATED, Json(res)))
        .map_err(|e| {
            tracing::error!("create_product error: {:?}", e);
            AppError::Internal
        })
}

/// PATCH /v1/products/:item_uid/status (인증 필요)
pub async fn update_product_status(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(item_uid): Path<String>,
    Json(req): Json<UpdateItemStateReq>,
) -> Result<StatusCode, AppError> {
    let seller_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let product_id =
        deobfuscate(&item_uid).map_err(|_| AppError::NotFound("Not found".to_string()))?;

    state
        .product_service
        .update_product_state(seller_id, product_id, req)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|e| {
            tracing::error!("update_product_status error: {:?}", e);
            match e {
                crate::service::product_service::ProductServiceError::Forbidden => {
                    AppError::Forbidden
                }
                crate::service::product_service::ProductServiceError::NotFound => {
                    AppError::NotFound("Not found".to_string())
                }
                _ => AppError::Internal,
            }
        })
}

/// PUT /v1/products/:item_uid (인증 필요)
pub async fn update_product(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(item_uid): Path<String>,
    Json(req): Json<UpdateItemReq>,
) -> Result<Json<ItemDetailRes>, AppError> {
    use validator::Validate;
    req.validate()
        .map_err(|_| AppError::BadRequest("Invalid request".to_string()))?;

    let seller_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let product_id =
        deobfuscate(&item_uid).map_err(|_| AppError::NotFound("Not found".to_string()))?;

    state
        .product_service
        .update_product(seller_id, product_id, req)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("update_product error: {:?}", e);
            match e {
                crate::service::product_service::ProductServiceError::Forbidden => {
                    AppError::Forbidden
                }
                crate::service::product_service::ProductServiceError::NotFound => {
                    AppError::NotFound("Not found".to_string())
                }
                _ => AppError::Internal,
            }
        })
}
