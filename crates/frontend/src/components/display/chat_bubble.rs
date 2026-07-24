// crates/frontend/src/components/display/chat_bubble.rs
// 목적: 채팅 말풍선 — is_mine에 따라 좌(상대방)/우(나) 정렬, 시간 표시

use leptos::prelude::*;
use shared::dto::chat_dto::ChatMessagePayload;

#[component]
pub fn ChatBubble(msg: ChatMessagePayload, is_mine: bool) -> impl IntoView {
    let bubble_class = if is_mine {
        "chat-bubble chat-bubble-mine"
    } else {
        "chat-bubble chat-bubble-other"
    };
    let wrapper_class = if is_mine {
        "chat-bubble-wrapper chat-bubble-wrapper-mine"
    } else {
        "chat-bubble-wrapper chat-bubble-wrapper-other"
    };

    view! {
        <div class=wrapper_class>
            <div class=bubble_class>
                <p class="chat-bubble-text">{msg.text.clone()}</p>
                <span class="chat-bubble-time">{msg.sent_at.clone()}</span>
            </div>
        </div>
    }
}
