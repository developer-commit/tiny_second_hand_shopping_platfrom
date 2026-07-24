// crates/frontend/src/components/display/empty_state.rs
// 목적: 빈 리스트 안내 컴포넌트 — 이모지 아이콘 + 메시지

use leptos::prelude::*;

#[component]
pub fn EmptyState(#[prop(into)] icon: String, #[prop(into)] message: String) -> impl IntoView {
    // StoredValue로 래핑하여 view 내에서 clone 없이 재사용 가능하게 함
    let icon = StoredValue::new(icon);
    let message = StoredValue::new(message);

    view! {
        <div class="empty-state" role="status" aria-label=move || message.get_value()>
            <span class="empty-state-icon">{move || icon.get_value()}</span>
            <p class="empty-state-message">{move || message.get_value()}</p>
        </div>
    }
}
