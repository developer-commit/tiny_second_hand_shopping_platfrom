// crates/frontend/src/models/product_model.rs
// 목적: 상품 상태 반응형 래퍼.
// shared::dto의 ItemDetailRes를 받아 Leptos RwSignal로 변환합니다.
// is_available Signal로 구매 버튼 표시 여부를 반응적으로 제어합니다.

use leptos::prelude::*;
use shared::dto::product_dto::{ItemDetailRes, ItemState};

#[derive(Clone, Debug)]
pub struct ProductUIState {
    pub item_uid: String,             // 라우팅/API 호출용 (불변)
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
