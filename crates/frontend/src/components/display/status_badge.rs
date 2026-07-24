// crates/frontend/src/components/display/status_badge.rs
// 목적: 상품 판매 상태 배지 — OnSale(green) / Reserved(orange) / Sold(gray)

use leptos::prelude::*;
use shared::dto::product_dto::ItemState;

#[component]
pub fn StatusBadge(state: ItemState) -> impl IntoView {
    let (label, class) = match state {
        ItemState::OnSale => ("판매중", "badge badge-success"),
        ItemState::Reserved => ("예약중", "badge badge-warning"),
        ItemState::Sold => ("판매완료", "badge badge-muted"),
    };

    view! {
        <span class=class>{label}</span>
    }
}
