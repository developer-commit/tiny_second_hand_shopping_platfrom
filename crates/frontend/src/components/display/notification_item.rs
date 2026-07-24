// crates/frontend/src/components/display/notification_item.rs
// 목적: 알림 목록 항목 — NotificationKind별 아이콘, 읽음/안읽음 스타일
// 클릭 시 kind에 따라 해당 페이지로 네비게이션합니다.

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use shared::dto::noti_dto::{NotificationKind, NotificationRes};

#[component]
pub fn NotificationItem(noti: NotificationRes) -> impl IntoView {
    let navigate = use_navigate();

    let (icon, _kind_label) = match noti.kind {
        NotificationKind::Chat         => ("💬", "채팅"),
        NotificationKind::EscrowUpdate => ("🔒", "에스크로"),
        NotificationKind::System       => ("🔔", "시스템"),
        NotificationKind::Warning      => ("⚠️", "경고"),
        NotificationKind::ReportResult => ("📋", "신고결과"),
    };

    let link_uid = noti.link_uid.clone();
    let kind = noti.kind.clone();
    let on_click = move |_| {
        let path = match &kind {
            NotificationKind::Chat         => "/chat".to_string(),
            NotificationKind::EscrowUpdate => link_uid
                .as_ref()
                .map(|uid| format!("/escrow/{}", uid))
                .unwrap_or_else(|| "/".to_string()),
            NotificationKind::System       => return,
            NotificationKind::Warning      => "/mypage".to_string(),
            NotificationKind::ReportResult => link_uid
                .as_ref()
                .map(|uid| format!("/products/{}", uid))
                .unwrap_or_else(|| "/".to_string()),
        };
        navigate(&path, Default::default());
    };

    let row_class = if noti.is_read {
        "noti-item noti-item-read"
    } else {
        "noti-item noti-item-unread"
    };

    view! {
        <div
            class=row_class
            on:click=on_click
            role="button"
            tabindex="0"
            aria-label=noti.content.clone()
        >
            <span class="noti-icon">{icon}</span>
            <div class="noti-body">
                <p class="noti-content">{noti.content.clone()}</p>
                <span class="noti-time">{noti.received_at.clone()}</span>
            </div>
            <Show when=move || !noti.is_read>
                <span class="noti-unread-dot" aria-label="읽지 않음"></span>
            </Show>
        </div>
    }
}
