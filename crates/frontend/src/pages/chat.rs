// crates/frontend/src/pages/chat.rs
// URL: /chat

use leptos::prelude::*;
use leptos_meta::Title;
use shared::dto::chat_dto::{ChatMessagePayload, SendMessageReq};

#[component]
pub fn ChatPage() -> impl IntoView {
    let messages = RwSignal::new(Vec::<ChatMessagePayload>::new());
    let msg_input = RwSignal::new(String::new());
    let active_room_uid = RwSignal::new(Option::<String>::None);

    let on_send = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if let Some(room_uid) = active_room_uid.get() {
            let req = SendMessageReq {
                room_uid,
                text: msg_input.get(),
            };
            todo!("WebSocket send → messages 업데이트")
        }
    };

    view! {
        <Title text="채팅"/>
        <main class="chat-page">
            <aside class="room-list">
                // todo!("채팅방 목록 표시")
            </aside>
            <section class="chat-window">
                <div class="messages">
                    <For
                        each=move || messages.get()
                        key=|m| m.msg_uid.clone()
                        children=move |msg| view! {
                            <div class="message">
                                <span class="sender">{msg.sender_uid.clone()}</span>
                                <span class="text">{msg.text.clone()}</span>
                            </div>
                        }
                    />
                </div>
                <form on:submit=on_send>
                    <input type="text" placeholder="메시지 입력..." on:input=move |e| msg_input.set(event_target_value(&e)) />
                    <button type="submit">"전송"</button>
                </form>
            </section>
        </main>
    }
}
