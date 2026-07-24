// crates/backend/src/handlers/chat_handler.rs
// 목적: WebSocket 채팅 핸들러 및 REST 채팅방/이력 핸들러.
// WebSocket 연결 수락 후 Redis Pub/Sub 메시지를 WebSocket으로 relay합니다.

use crate::state::AppState;
use crate::utils::auth::Claims;
use crate::utils::error::AppError;
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use shared::dto::chat_dto::{ChatMessagePayload, ChatRoomRes, CreateChatRoomReq};

use crate::utils::security::deobfuscate;

/// POST /v1/chat/rooms (인증 필요) — 1:1 채팅방 생성/조회
pub async fn create_or_get_room(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<CreateChatRoomReq>,
) -> Result<Json<ChatRoomRes>, AppError> {
    let user_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    state
        .chat_service
        .get_or_create_room(user_id, req)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("create_or_get_room error: {:?}", e);
            match e {
                crate::service::chat_service::ChatServiceError::Forbidden => AppError::Forbidden,
                _ => AppError::Internal,
            }
        })
}

/// GET /v1/chat/rooms (인증 필요) — 사용자의 모든 채팅방 조회
pub async fn list_rooms(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<ChatRoomRes>>, AppError> {
    let user_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    state
        .chat_service
        .get_user_rooms(user_id)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("list_rooms error: {:?}", e);
            match e {
                crate::service::chat_service::ChatServiceError::Forbidden => AppError::Forbidden,
                _ => AppError::Internal,
            }
        })
}

/// GET /v1/chat/rooms/:room_uid/history (인증 필요) — 메시지 이력
pub async fn get_chat_history(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(room_uid): Path<String>,
) -> Result<Json<Vec<ChatMessagePayload>>, AppError> {
    let user_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let room_id =
        deobfuscate(&room_uid).map_err(|_| AppError::NotFound("Not found".to_string()))?;

    state
        .chat_service
        .get_history(user_id, room_id, None)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("get_chat_history error: {:?}", e);
            match e {
                crate::service::chat_service::ChatServiceError::Forbidden => AppError::Forbidden,
                _ => AppError::Internal,
            }
        })
}

/// POST /v1/chat/rooms/:room_uid/messages (인증 필요) — 메시지 전송
pub async fn send_message(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(_room_uid): Path<String>,
    Json(req): Json<shared::dto::chat_dto::SendMessageReq>,
) -> Result<Json<ChatMessagePayload>, AppError> {
    let user_id = deobfuscate(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    state
        .chat_service
        .send_message(user_id, req)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("send_message error: {:?}", e);
            match e {
                crate::service::chat_service::ChatServiceError::Forbidden => AppError::Forbidden,
                crate::service::chat_service::ChatServiceError::RoomNotFound => {
                    AppError::NotFound("Room not found".to_string())
                }
                _ => AppError::Internal,
            }
        })
}

/// GET /v1/chat/ws (WebSocket 업그레이드 — JWT는 쿼리 파라미터로 전달)
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, claims))
}

async fn handle_socket(socket: axum::extract::ws::WebSocket, state: AppState, claims: Claims) {
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};
    use redis::AsyncCommands;

    let user_id = match deobfuscate(&claims.sub) {
        Ok(id) => id,
        Err(_) => return,
    };

    tracing::info!("WS connected for user_id: {}", user_id);

    let mut pubsub = match state.redis_client.get_async_pubsub().await {
        Ok(ps) => ps,
        Err(e) => {
            tracing::error!(
                "Failed to get redis pubsub for user_id {}: {:?}",
                user_id,
                e
            );
            return;
        }
    };

    let channel_name = format!("user_{}", user_id);
    if let Err(e) = pubsub.subscribe(&channel_name).await {
        tracing::error!("Failed to subscribe to channel {}: {:?}", channel_name, e);
        return;
    }

    tracing::info!("WS subscribed to {}", channel_name);

    let (mut sender, mut receiver) = socket.split();
    let mut pubsub_stream = pubsub.into_on_message();

    loop {
        tokio::select! {
            msg_opt = pubsub_stream.next() => {
                if let Some(msg) = msg_opt {
                    if let Ok(payload) = msg.get_payload::<String>() {
                        if sender.send(Message::Text(payload.into())).await.is_err() {
                            tracing::info!("WS client disconnected while sending");
                            break;
                        }
                    }
                } else {
                    tracing::info!("PubSub stream closed");
                    break;
                }
            }
            client_msg_opt = receiver.next() => {
                if let Some(Ok(msg)) = client_msg_opt {
                    // We only care about Close messages from client
                    if let Message::Close(_) = msg {
                        tracing::info!("WS client requested close");
                        break;
                    }
                } else {
                    tracing::info!("WS client disconnected");
                    break;
                }
            }
        }
    }
}
