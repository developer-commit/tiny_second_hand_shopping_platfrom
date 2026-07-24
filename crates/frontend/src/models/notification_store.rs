// crates/frontend/src/models/notification_store.rs
// 목적: 알림 전역 저장소 — WebSocket/SSE에서 수신한 알림을 반응형으로 관리.

use gloo_net::http::Request;
use leptos::prelude::*;
use shared::dto::noti_dto::NotificationRes;

const API_BASE_URL: &str = "/v1";

#[derive(Clone, Copy, Debug)]
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

    pub fn set_notifications(&self, notis: Vec<NotificationRes>) {
        let unread = notis.iter().filter(|n| !n.is_read).count();
        self.notifications.set(notis);
        self.unread_count.set(unread);
    }
}

/// GET /notifications - 알림 목록 조회
pub async fn fetch_notifications(token: &str) -> Result<Vec<NotificationRes>, String> {
    let url = format!("{}/notifications", API_BASE_URL);
    let res = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        let err_msg = err_res
            .map(|e| e.message)
            .unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
        return Err(err_msg);
    }

    res.json().await.map_err(|e| e.to_string())
}

/// PATCH /notifications/{noti_uid}/read - 알림 읽음 처리 API
pub async fn mark_notification_read(token: &str, noti_uid: &str) -> Result<(), String> {
    let url = format!("{}/notifications/{}/read", API_BASE_URL, noti_uid);
    let res = Request::patch(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        let err_msg = err_res
            .map(|e| e.message)
            .unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
        return Err(err_msg);
    }

    Ok(())
}
