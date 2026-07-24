// crates/frontend/src/pages/notifications.rs
// URL: /notifications

use leptos::prelude::*;
use leptos_meta::Title;
use crate::components::{
    layout::PageContainer,
    display::{NotificationItem, EmptyState},
};
use crate::models::{
    notification_store::{NotificationStore, fetch_notifications},
    auth_model::AuthStore,
};
use gloo_net::http::Request;

async fn mark_notification_read(token: &str, noti_uid: &str) -> Result<(), String> {
    let url = format!("{}/notifications/{}/read", "/v1", noti_uid);
    let res = Request::patch(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        return Err("읽음 처리 실패".into());
    }
    Ok(())
}

#[component]
pub fn NotificationsPage() -> impl IntoView {
    let auth_store = expect_context::<AuthStore>();
    let noti_store = expect_context::<NotificationStore>();
    
    let token = Signal::derive(move || auth_store.bearer_header().unwrap_or_default());
    
    let notis_res = LocalResource::new(move || {
        let t = token.get();
        async move {
            if t.is_empty() { return Err("로그인이 필요합니다.".to_string()); }
            fetch_notifications(&t).await
        }
    });

    let noti_store_effect = noti_store.clone();
    Effect::new(move |_| {
        if let Some(res) = notis_res.get() {
            if let Ok(notis) = &*res {
                noti_store_effect.set_notifications(notis.clone());
            }
        }
    });

    let mark_read_action = Action::new_local(move |noti_uid: &String| {
        let t = token.get();
        let uid = noti_uid.clone();
        async move {
            mark_notification_read(&t, &uid).await
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(_)) = mark_read_action.value().get() {
            notis_res.refetch(); // Refresh list to reflect read status
        }
    });

    view! {
        <Title text="알림 내역"/>
        <PageContainer title="알림 내역">
            <div class="notifications-page" style="max-width: 600px; margin: 0 auto; padding-top: 2rem; display: flex; flex-direction: column; gap: 2rem;">
                <h1 style="font-size: 1.5rem; font-weight: bold;">"알림 내역"</h1>
                
                <Suspense fallback=move || view! { <p>"알림을 불러오는 중..."</p> }>
                    {move || notis_res.get().map(|res| match &*res {
                        Ok(_) => {
                            let notis = noti_store.notifications.get();
                            if notis.is_empty() {
                                view! { <EmptyState icon="🔔" message="새로운 알림이 없습니다." /> }.into_any()
                            } else {
                                view! {
                                    <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                                        {notis.into_iter().map(|noti| {
                                            let uid = noti.noti_uid.clone();
                                            view! {
                                                <div on:click=move |_| {
                                                    mark_read_action.dispatch(uid.clone());
                                                } style="cursor: pointer;">
                                                    <NotificationItem noti=noti />
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            }
                        }
                        Err(e) => view! { <div style="color: red;">{e.to_string()}</div> }.into_any(),
                    })}
                </Suspense>
            </div>
        </PageContainer>
    }
}
