// crates/frontend/src/models/notification_store.rs
// 목적: 알림 전역 저장소 — WebSocket/SSE에서 수신한 알림을 반응형으로 관리.

use leptos::prelude::*;
use shared::dto::noti_dto::NotificationRes;

#[derive(Clone, Debug)]
pub struct NotificationStore {
    pub unread_count: RwSignal<usize>,
    pub notifications: RwSignal<Vec<NotificationRes>>,
    pub has_unread: Signal<bool>,
}

impl NotificationStore {
    pub fn new() -> Self {
        let unread_count = RwSignal::new(0_usize);
        let has_unread = Signal::derive(move || unread_count.get() > 0);

        NotificationStore {
            unread_count,
            notifications: RwSignal::new(Vec::new()),
            has_unread,
        }
    }

    /// 새 알림 추가 (WebSocket 메시지 수신 시)
    pub fn push(&self, noti: NotificationRes) {
        self.notifications.update(|list| list.insert(0, noti));
        self.unread_count.update(|c| *c += 1);
    }

    /// 알림 읽음 처리
    pub fn mark_read(&self, noti_uid: &str) {
        self.notifications.update(|list| {
            if let Some(n) = list.iter_mut().find(|n| n.noti_uid == noti_uid) {
                if !n.is_read {
                    n.is_read = true;
                    self.unread_count.update(|c| *c = c.saturating_sub(1));
                }
            }
        });
    }
}
