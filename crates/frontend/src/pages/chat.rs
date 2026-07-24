// crates/frontend/src/pages/chat.rs
// URL: /chat

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;
use shared::dto::chat_dto::SendMessageReq;
use crate::components::{
    layout::PageContainer,
    display::ChatBubble,
    input::FormInput,
    feedback::{AppButton, ButtonVariant},
};
use crate::models::{
    chat_store::{ChatStore, fetch_chat_rooms, fetch_chat_messages},
    auth_model::AuthStore,
};
use gloo_net::http::Request;

async fn send_message_http(token: &str, req: &SendMessageReq) -> Result<(), String> {
    let url = format!("{}/chat/rooms/{}/messages", "/v1", req.room_uid);
    let res = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        return Err("메시지 전송 실패".into());
    }
    Ok(())
}

#[component]
pub fn ChatPage() -> impl IntoView {
    let auth_store = expect_context::<AuthStore>();
    let chat_store = expect_context::<ChatStore>();
    
    let token = Signal::derive(move || auth_store.bearer_header().unwrap_or_default());
    let msg_input = RwSignal::new(String::new());

    let rooms_res = LocalResource::new(move || {
        let t = token.get();
        async move {
            if t.is_empty() { return Err("로그인이 필요합니다.".to_string()); }
            fetch_chat_rooms(&t).await
        }
    });
    
    let active_room_uid = chat_store.active_room;
    let messages = chat_store.messages;
    
    let query = use_query_map();
    Effect::new(move |_| {
        if let Some(uid) = query.with(|q| q.get("room_uid")) {
            chat_store.set_active_room(Some(uid));
        }
    });

    Effect::new(move |_| {
        let room_uid = active_room_uid.get();
        let t = token.get();
        if let Some(uid) = room_uid {
            if !t.is_empty() {
                leptos::task::spawn_local(async move {
                    if let Ok(msgs) = fetch_chat_messages(&t, &uid).await {
                        chat_store.set_messages(msgs);
                    }
                });
            }
        }
    });

    let is_sending = RwSignal::new(false);

    Effect::new(move |_| {
        let t = token.get();
        if t.is_empty() { return; }
        
        let window = web_sys::window().unwrap();
        let host = window.location().host().unwrap();
        let protocol = if window.location().protocol().unwrap() == "https:" { "wss:" } else { "ws:" };
        let ws_url = format!("{}//{}/v1/chat/ws?token={}", protocol, host, t);
        
        if let Ok(ws) = web_sys::WebSocket::new(&ws_url) {
            let chat_store_clone = chat_store.clone();
            let onmessage_callback = wasm_bindgen::closure::Closure::<dyn FnMut(_)>::new(move |e: web_sys::MessageEvent| {
                use wasm_bindgen::JsCast;
                if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                    let text: String = txt.into();
                    if let Ok(payload) = serde_json::from_str::<shared::dto::chat_dto::ChatMessagePayload>(&text) {
                        chat_store_clone.upsert_message(payload.clone());
                        
                        if let Some(Ok(rooms)) = rooms_res.get_untracked().as_deref() {
                            if !rooms.iter().any(|r| r.room_uid == payload.room_uid) {
                                rooms_res.refetch();
                            }
                        }
                    }
                }
            });
            
            use wasm_bindgen::JsCast;
            ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            onmessage_callback.forget();
            
            on_cleanup(move || {
                let _ = ws.close();
            });
        }
    });

    let on_send = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if let Some(room_uid) = active_room_uid.get() {
            let text = msg_input.get();
            if text.is_empty() { return; }
            
            let req = SendMessageReq {
                room_uid: room_uid.clone(),
                text: text.clone(),
            };
            
            let fake_uid = format!("pending-{}", js_sys::Date::now() as i64);
            let fake_msg = shared::dto::chat_dto::ChatMessagePayload {
                msg_uid: fake_uid.clone(),
                room_uid: room_uid.clone(),
                sender_uid: auth_store.current_user.get().map(|u| u.user_uid).unwrap_or_default(),
                text: text.clone(),
                read_status: false,
                sent_at: chrono::Utc::now().to_rfc3339(),
            };
            chat_store.push_message(fake_msg);
            msg_input.set(String::new());
            
            let t = token.get_untracked();
            let chat_store_clone = chat_store.clone();
            
            is_sending.set(true);
            leptos::task::spawn_local(async move {
                if send_message_http(&t, &req).await.is_err() {
                    chat_store_clone.messages.update(|msgs| {
                        msgs.retain(|m| m.msg_uid != fake_uid);
                    });
                }
                is_sending.set(false);
            });
        }
    };

    view! {
        <Title text="채팅"/>
        <PageContainer title="채팅">
            <div class="chat-page" style="display: flex; height: calc(100vh - 120px); gap: 1.5rem; padding: 1.5rem; background-color: #f8f9fa; border: 2px solid #e9ecef; border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.05);">
                <aside class="room-list" style="width: 300px; border-right: 2px solid #dee2e6; overflow-y: auto; background: #ffffff; border-radius: 8px; box-shadow: inset 0 0 5px rgba(0,0,0,0.05);">
                    <h2 style="font-size: 1.5rem; font-weight: bold; margin-bottom: 1rem; padding: 1rem; border-bottom: 1px solid #dee2e6; background: #e9ecef; color: #495057;">"채팅방 목록"</h2>
                    <Suspense fallback=move || view! { <p style="padding: 1rem;">"채팅방 불러오는 중..."</p> }>
                        {move || rooms_res.get().map(|res| match &*res {
                            Ok(rooms) => {
                                let rooms = rooms.clone();
                                view! {
                                    <div style="display: flex; flex-direction: column; gap: 0.5rem; padding: 0.5rem;">
                                        {rooms.into_iter().map(|room| {
                                        let is_active = active_room_uid.get().as_deref() == Some(&room.room_uid);
                                        let bg = if is_active { "#e7f1ff" } else { "#ffffff" };
                                        let border = if is_active { "2px solid #0d6efd" } else { "1px solid #dee2e6" };
                                        view! {
                                            <div 
                                                style=format!("padding: 1rem; border-radius: 8px; cursor: pointer; background: {}; border: {}; transition: background 0.2s;", bg, border)
                                                on:click=move |_| chat_store.set_active_room(Some(room.room_uid.clone()))
                                            >
                                                <div style="display: flex; justify-content: space-between; align-items: center;">
                                                    <div style="font-weight: 600; color: #212529;">
                                                        {
                                                            let me = auth_store.current_user.get().map(|u| u.user_uid);
                                                            room.participant_uids.iter().find(|&id| Some(id) != me.as_ref()).cloned().unwrap_or_else(|| "상대방".to_string())
                                                        }
                                                    </div>
                                                    <div style="font-size: 0.75rem; color: #0d6efd; font-weight: bold; background: #e7f1ff; padding: 0.1rem 0.5rem; border-radius: 12px;">
                                                        {room.product_name.clone().unwrap_or_else(|| "일반 채팅".to_string())}
                                                    </div>
                                                </div>
                                                <div style="font-size: 0.875rem; color: #6c757d; text-overflow: ellipsis; overflow: hidden; white-space: nowrap; margin-top: 0.25rem;">
                                                    "새로운 메시지를 확인해보세요."
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                    </div>
                                }.into_any()
                            },
                            Err(e) => view! { <div style="color: red; padding: 1rem;">{e.to_string()}</div> }.into_any(),
                        })}
                    </Suspense>
                </aside>
                
                <section class="chat-window" style="flex: 1; display: flex; flex-direction: column; background: #ffffff; border: 2px solid #dee2e6; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 6px rgba(0,0,0,0.05);">
                    {move || if active_room_uid.get().is_some() {
                        view! {
                            <div style="flex: 1; overflow-y: auto; padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem; background: #f1f3f5;">
                                <For
                                    each=move || messages.get()
                                    key=|m| m.msg_uid.clone()
                                    children=move |msg| {
                                        let is_mine = msg.sender_uid == auth_store.current_user.get().map(|u| u.user_uid).unwrap_or_default();
                                        view! {
                                            <div style=format!("display: flex; width: 100%; justify-content: {}", if is_mine { "flex-end" } else { "flex-start" })>
                                                <div style=format!("padding: 0.75rem 1rem; border-radius: 16px; max-width: 70%; {}", if is_mine { "background-color: #0d6efd; color: white; border-bottom-right-radius: 4px;" } else { "background-color: #ffffff; color: #212529; border: 1px solid #dee2e6; border-bottom-left-radius: 4px;" })>
                                                    <ChatBubble msg=msg is_mine=is_mine />
                                                </div>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                            
                            <form on:submit=on_send style="display: flex; gap: 1rem; padding: 1rem; border-top: 2px solid #dee2e6; background: #ffffff; align-items: center;">
                                <div style="flex: 1;">
                                    <FormInput
                                        label=""
                                        placeholder="메시지 입력..."
                                        input_type="text"
                                        signal=msg_input
                                        error=Signal::derive(|| None)
                                    />
                                </div>
                                <div style="padding-top: 0.5rem;">
                                    <AppButton
                                        variant=ButtonVariant::Primary
                                        loading=Signal::derive(move || is_sending.get())
                                        disabled=Signal::derive(move || msg_input.get().is_empty())
                                        button_type="submit"
                                    >
                                        "전송"
                                    </AppButton>
                                </div>
                            </form>
                        }.into_any()
                    } else {
                        view! {
                            <div style="flex: 1; display: flex; align-items: center; justify-content: center; color: #adb5bd; font-size: 1.25rem; font-weight: 500; background: #f8f9fa;">
                                "채팅방을 선택해주세요."
                            </div>
                        }.into_any()
                    }}
                </section>
            </div>
        </PageContainer>
    }
}
