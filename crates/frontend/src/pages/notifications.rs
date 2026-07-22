// crates/frontend/src/pages/notifications.rs
// URL: /notifications

use leptos::prelude::*;
use leptos_meta::Title;
use shared::dto::noti_dto::NotificationKind;

#[component]
pub fn NotificationsPage() -> impl IntoView {
    // todo!("NotificationStore context에서 notifications 읽기")
    view! {
        <Title text="알림 내역"/>
        <main class="notifications-page">
            <h1>"알림 내역"</h1>
            // todo!("notifications 목록 렌더링 + kind에 따른 아이콘 분기")
            <p>"알림이 없습니다."</p>
        </main>
    }
}
