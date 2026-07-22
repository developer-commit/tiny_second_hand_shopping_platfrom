// crates/backend/src/handlers/chat_handler.rs
// 목적: WebSocket 채팅 핸들러 및 REST 채팅방/이력 핸들러.
// WebSocket 연결 수락 후 Redis Pub/Sub 메시지를 WebSocket으로 relay합니다.

use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::{IntoResponse, Json},
    http::StatusCode,
};
use shared::dto::chat_dto::{ChatMessagePayload, ChatRoomRes, CreateChatRoomReq};
use crate::state::AppState;
use crate::utils::auth::Claims;

/// POST /v1/chat/rooms (인증 필요) — 1:1 채팅방 생성/조회
pub async fn create_or_get_room(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<CreateChatRoomReq>,
) -> Result<Json<ChatRoomRes>, StatusCode> {
    todo!("claims → user_id → state.chat_service.get_or_create_room(user_id, req).await → Json")
}

/// GET /v1/chat/rooms/:room_uid/history (인증 필요) — 메시지 이력
pub async fn get_chat_history(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(room_uid): Path<String>,
) -> Result<Json<Vec<ChatMessagePayload>>, StatusCode> {
    todo!("claims → user_id, deobfuscate(room_uid) → state.chat_service.get_history(user_id, room_id, None).await → Json")
}

/// GET /v1/chat/ws (WebSocket 업그레이드 — JWT는 쿼리 파라미터로 전달)
/// WebSocket 핸드셰이크: HTTP → WS 업그레이드
/// 연결 후 흐름: 메시지 수신 → chat_service.send_message() → Redis 발행 → 채널 구독자에게 브로드캐스트
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, claims))
}

async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    state: AppState,
    claims: Claims,
) {
    todo!("소켓 분할(split) → 수신 루프(SendMessageReq 파싱 → chat_service.send_message) + Redis SUB 루프(수신 메시지 → 소켓 전송)")
}
