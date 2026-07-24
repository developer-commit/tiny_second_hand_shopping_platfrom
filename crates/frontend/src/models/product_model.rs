// crates/frontend/src/models/product_model.rs
// 목적: 상품 상태 반응형 래퍼.
// shared::dto의 ItemDetailRes를 받아 Leptos RwSignal로 변환합니다.
// is_available Signal로 구매 버튼 표시 여부를 반응적으로 제어합니다.

use leptos::prelude::*;
use shared::dto::product_dto::{ItemDetailRes, ItemState, ItemSummaryRes, ProductSearchQuery};
use gloo_net::http::Request;

const API_BASE_URL: &str = "/v1";

#[derive(Clone, Debug)]
pub struct ProductUIState {
    pub item_uid: String,             // 라우팅/API 호출용 (불변)
    pub seller_trust_score: RwSignal<Option<f64>>,
    pub heading: RwSignal<String>,
    pub asking_price: RwSignal<f64>,
    pub current_state: RwSignal<ItemState>,
    pub image_urls: RwSignal<Vec<String>>,
    pub tags: RwSignal<Vec<String>>,
    /// 구매 가능 여부 파생 시그널 (on_sale 상태일 때만 true)
    pub is_available: Signal<bool>,
    /// 상태 배지 텍스트
    pub state_badge: Signal<String>,
}

impl ProductUIState {
    pub fn from_dto(dto: ItemDetailRes) -> Self {
        let state_signal = RwSignal::new(dto.current_state);
        let is_available = Signal::derive(move || state_signal.get() == ItemState::OnSale);
        let state_badge = Signal::derive(move || match state_signal.get() {
            ItemState::OnSale   => "판매중".to_string(),
            ItemState::Reserved => "예약중".to_string(),
            ItemState::Sold     => "판매완료".to_string(),
        });

        ProductUIState {
            item_uid: dto.item_uid,
            seller_trust_score: RwSignal::new(dto.seller_trust_score),
            heading: RwSignal::new(dto.heading),
            asking_price: RwSignal::new(dto.asking_price),
            current_state: state_signal,
            image_urls: RwSignal::new(dto.image_urls),
            tags: RwSignal::new(dto.tags),
            is_available,
            state_badge,
        }
    }
}

/// GET /products - 상품 목록 조회 API
pub async fn fetch_products(query: ProductSearchQuery) -> Result<Vec<ItemSummaryRes>, String> {
    let mut url = format!("{}/products", API_BASE_URL);
    let mut params = Vec::new();
    
    if let Some(k) = query.keyword { params.push(format!("keyword={}", k)); }
    if let Some(c) = query.group_category { params.push(format!("group_category={}", c)); }
    if let Some(t) = query.item_tag { params.push(format!("item_tag={}", t)); }
    if let Some(p) = query.page { params.push(format!("page={}", p)); }
    if let Some(ps) = query.page_size { params.push(format!("page_size={}", ps)); }
    
    if !params.is_empty() {
        url = format!("{}?{}", url, params.join("&"));
    }

    let res = Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        let err_msg = err_res.map(|e| e.message).unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
        return Err(err_msg);
    }

    res.json().await.map_err(|e| e.to_string())
}

/// GET /products/{item_uid} - 상품 상세 조회 API
pub async fn fetch_product_detail(item_uid: String) -> Result<ItemDetailRes, String> {
    let url = format!("{}/products/{}", API_BASE_URL, item_uid);
    let res = Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        let err_msg = err_res.map(|e| e.message).unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
        return Err(err_msg);
    }

    res.json().await.map_err(|e| e.to_string())
}

/// 상품 목록 리소스 생성 헬퍼
pub fn create_products_resource(query: impl Fn() -> ProductSearchQuery + Send + Sync + 'static) -> LocalResource<Result<Vec<ItemSummaryRes>, String>> {
    LocalResource::new(move || {
        let q = query();
        async move { fetch_products(q).await }
    })
}
