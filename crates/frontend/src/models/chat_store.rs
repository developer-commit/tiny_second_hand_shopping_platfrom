// crates/frontend/src/models/chat_store.rs
// 목적: 채팅방 및 메시지 전역 상태 관리

use leptos::prelude::*;
use shared::dto::chat_dto::{ChatRoomRes, ChatMessagePayload, CreateChatRoomReq};
use gloo_net::http::Request;

const API_BASE_URL: &str = "/v1";

#[derive(Clone, Copy, Debug)]
pub struct ChatStore {
    pub rooms: RwSignal<Vec<ChatRoomRes>>,
    pub active_room: RwSignal<Option<String>>,
    pub messages: RwSignal<Vec<ChatMessagePayload>>,
}

impl ChatStore {
    pub fn new() -> Self {
        ChatStore {
            rooms: RwSignal::new(Vec::new()),
            active_room: RwSignal::new(None),
            messages: RwSignal::new(Vec::new()),
        }
    }

    pub fn set_rooms(&self, rooms: Vec<ChatRoomRes>) {
        self.rooms.set(rooms);
    }

    pub fn set_active_room(&self, room_uid: Option<String>) {
        self.active_room.set(room_uid);
        // Change room means messages should be cleared before fetching new ones
        self.messages.set(Vec::new());
    }

    pub fn set_messages(&self, messages: Vec<ChatMessagePayload>) {
        self.messages.set(messages);
    }

    pub fn push_message(&self, msg: ChatMessagePayload) {
        if Some(msg.room_uid.clone()) == self.active_room.get() {
            self.messages.update(|msgs| msgs.push(msg));
        }
    }

    pub fn upsert_message(&self, payload: ChatMessagePayload) {
        if Some(payload.room_uid.clone()) == self.active_room.get() {
            self.messages.update(|msgs| {
                if let Some(pos) = msgs.iter().position(|m| m.msg_uid.starts_with("pending-") && m.text == payload.text && m.sender_uid == payload.sender_uid) {
                    msgs[pos] = payload;
                } else {
                    if !msgs.iter().any(|m| m.msg_uid == payload.msg_uid) {
                        msgs.push(payload);
                    }
                }
            });
        }
    }
}

/// GET /chat/rooms - 채팅방 목록 조회
pub async fn fetch_chat_rooms(token: &str) -> Result<Vec<ChatRoomRes>, String> {
    let url = format!("{}/chat/rooms", API_BASE_URL);
    let res = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        let err_msg = err_res.map(|e| e.message).unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
        return Err(err_msg);
    }

    res.json().await.map_err(|e| e.to_string())
}

/// GET /chat/rooms/{room_uid}/history - 메시지 내역 조회
pub async fn fetch_chat_messages(token: &str, room_uid: &str) -> Result<Vec<ChatMessagePayload>, String> {
    let url = format!("{}/chat/rooms/{}/history", API_BASE_URL, room_uid);
    let res = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        let err_msg = err_res.map(|e| e.message).unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
        return Err(err_msg);
    }

    res.json().await.map_err(|e| e.to_string())
}

/// POST /chat/rooms - 채팅방 생성
pub async fn create_chat_room(token: &str, req: CreateChatRoomReq) -> Result<ChatRoomRes, String> {
    let url = format!("{}/chat/rooms", API_BASE_URL);
    let res = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        let err_msg = err_res.map(|e| e.message).unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
        return Err(err_msg);
    }

    res.json().await.map_err(|e| e.to_string())
}
