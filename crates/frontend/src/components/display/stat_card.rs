// crates/frontend/src/components/display/stat_card.rs
// 목적: 관리자 대시보드 통계 수치 카드

use leptos::prelude::*;

#[component]
pub fn StatCard(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(into)] icon: String,
) -> impl IntoView {
    view! {
        <div class="stat-card">
            <span class="stat-card-icon">{icon}</span>
            <div class="stat-card-body">
                <span class="stat-card-value">{value}</span>
                <span class="stat-card-label">{label}</span>
            </div>
        </div>
    }
}
