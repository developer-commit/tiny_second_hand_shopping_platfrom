// crates/backend/src/service/chat_service.rs
// 목적: 채팅방 관리 및 메시지 저장/발행 비즈니스 로직.

use crate::service::traits::{ChatServiceTrait, NotificationServiceTrait};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use shared::dto::chat_dto::{ChatMessagePayload, ChatRoomRes, CreateChatRoomReq, SendMessageReq};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChatServiceError {
    #[error("채팅방을 찾을 수 없습니다.")]
    RoomNotFound,
    #[error("채팅방 접근 권한이 없습니다.")]
    Forbidden,
    #[error("내부 서버 오류: {0}")]
    Internal(String),
}

pub struct ChatService {
    db: DatabaseConnection,
    pubsub: Arc<dyn crate::ports::pubsub_port::PubSubPort>,
    notification_service: Arc<dyn NotificationServiceTrait>,
}

impl ChatService {
    pub fn new(
        db: DatabaseConnection,
        pubsub: Arc<dyn crate::ports::pubsub_port::PubSubPort>,
        notification_service: Arc<dyn NotificationServiceTrait>,
    ) -> Self {
        ChatService {
            db,
            pubsub,
            notification_service,
        }
    }
}

#[async_trait]
impl ChatServiceTrait for ChatService {
    async fn get_or_create_room(
        &self,
        user_id: i64,
        req: CreateChatRoomReq,
    ) -> Result<ChatRoomRes, ChatServiceError> {
        use crate::db::entity::{chat_participant, chat_room};
        use crate::utils::security::{deobfuscate, obfuscate};
        use sea_orm::{
            ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait,
        };

        let target_user_id = deobfuscate(&req.partner_uid)
            .map_err(|_| ChatServiceError::Internal("Invalid partner".to_string()))?;
        let item_id = deobfuscate(&req.item_uid)
            .map_err(|_| ChatServiceError::Internal("Invalid item".to_string()))?;

        // 1. Check if room exists for this product and these two participants
        let existing_rooms = chat_room::Entity::find()
            .filter(chat_room::Column::ProductId.eq(item_id))
            .all(&self.db)
            .await
            .map_err(|e| ChatServiceError::Internal(e.to_string()))?;

        for room in existing_rooms {
            let participants = chat_participant::Entity::find()
                .filter(chat_participant::Column::RoomId.eq(room.id))
                .all(&self.db)
                .await
                .map_err(|e| ChatServiceError::Internal(e.to_string()))?;

            let participant_ids: Vec<i64> = participants.iter().map(|p| p.user_id).collect();
            if participant_ids.contains(&user_id) && participant_ids.contains(&target_user_id) {
                let product_name = if let Ok(Some(prod)) =
                    crate::db::entity::product::Entity::find_by_id(item_id)
                        .one(&self.db)
                        .await
                {
                    Some(prod.title)
                } else {
                    None
                };
                return Ok(ChatRoomRes {
                    room_uid: obfuscate(room.id).unwrap_or_default(),
                    room_kind: shared::dto::chat_dto::RoomKind::Private,
                    item_uid: Some(req.item_uid.clone()),
                    product_name,
                    participant_uids: vec![
                        req.partner_uid.clone(),
                        obfuscate(user_id).unwrap_or_default(),
                    ],
                    created_at: room.created_at.to_rfc3339(),
                });
            }
        }

        // 2. Create new room if not exists
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| ChatServiceError::Internal(e.to_string()))?;

        let new_room = chat_room::ActiveModel {
            product_id: Set(Some(item_id)),
            room_type: Set("private".to_string()),
            created_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };
        let inserted_room = new_room
            .insert(&txn)
            .await
            .map_err(|e| ChatServiceError::Internal(e.to_string()))?;

        let p1 = chat_participant::ActiveModel {
            room_id: Set(inserted_room.id),
            user_id: Set(user_id),
            joined_at: Set(Some(chrono::Utc::now().into())),
        };
        let p2 = chat_participant::ActiveModel {
            room_id: Set(inserted_room.id),
            user_id: Set(target_user_id),
            joined_at: Set(Some(chrono::Utc::now().into())),
        };

        p1.insert(&txn)
            .await
            .map_err(|e| ChatServiceError::Internal(e.to_string()))?;
        p2.insert(&txn)
            .await
            .map_err(|e| ChatServiceError::Internal(e.to_string()))?;

        txn.commit()
            .await
            .map_err(|e| ChatServiceError::Internal(e.to_string()))?;

        let product_name = if let Ok(Some(prod)) =
            crate::db::entity::product::Entity::find_by_id(item_id)
                .one(&self.db)
                .await
        {
            Some(prod.title)
        } else {
            None
        };

        Ok(ChatRoomRes {
            room_uid: obfuscate(inserted_room.id).unwrap_or_default(),
            room_kind: shared::dto::chat_dto::RoomKind::Private,
            item_uid: Some(req.item_uid),
            product_name,
            participant_uids: vec![req.partner_uid, obfuscate(user_id).unwrap_or_default()],
            created_at: inserted_room.created_at.to_rfc3339(),
        })
    }

    async fn send_message(
        &self,
        sender_id: i64,
        req: SendMessageReq,
    ) -> Result<ChatMessagePayload, ChatServiceError> {
        use crate::db::entity::chat_message;
        use crate::utils::security::{deobfuscate, obfuscate};
        use sea_orm::{ActiveModelTrait, Set};

        let room_id = deobfuscate(&req.room_uid).map_err(|_| ChatServiceError::RoomNotFound)?;

        let msg = chat_message::ActiveModel {
            room_id: Set(room_id),
            sender_id: Set(sender_id),
            message: Set(req.text.clone()),
            is_read: Set(false),
            created_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };

        let inserted = msg
            .insert(&self.db)
            .await
            .map_err(|e| ChatServiceError::Internal(e.to_string()))?;

        let payload = ChatMessagePayload {
            msg_uid: obfuscate(inserted.id).unwrap_or_default(),
            room_uid: req.room_uid.clone(),
            sender_uid: obfuscate(sender_id).unwrap_or_default(),
            text: req.text,
            read_status: false,
            sent_at: inserted.created_at.to_rfc3339(),
        };

        if let Ok(payload_json) = serde_json::to_string(&payload) {
            // Fetch all participants to publish to their individual channels
            use crate::db::entity::chat_participant;
            use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

            let all_participants = chat_participant::Entity::find()
                .filter(chat_participant::Column::RoomId.eq(room_id))
                .all(&self.db)
                .await
                .unwrap_or_default();

            for p in all_participants {
                let channel = format!("user_{}", p.user_id);
                let _ = self.pubsub.publish(&channel, &payload_json).await;

                // Notify only the recipient via notification_service
                if p.user_id != sender_id {
                    let _ = self
                        .notification_service
                        .send_notification(
                            p.user_id,
                            "chat_message",
                            Some(room_id),
                            "새로운 채팅 메시지가 도착했습니다.",
                        )
                        .await;
                }
            }
        }

        Ok(payload)
    }

    async fn get_history(
        &self,
        user_id: i64,
        room_id: i64,
        before_msg_id: Option<i64>,
    ) -> Result<Vec<ChatMessagePayload>, ChatServiceError> {
        use crate::db::entity::chat_message;
        use crate::utils::security::obfuscate;
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

        let mut filter =
            chat_message::Entity::find().filter(chat_message::Column::RoomId.eq(room_id));
        if let Some(id) = before_msg_id {
            filter = filter.filter(chat_message::Column::Id.lt(id));
        }

        let messages = filter
            .order_by_desc(chat_message::Column::CreatedAt)
            .limit(50)
            .all(&self.db)
            .await
            .map_err(|e| ChatServiceError::Internal(e.to_string()))?;

        let mut result = Vec::new();
        for m in messages {
            result.push(ChatMessagePayload {
                msg_uid: obfuscate(m.id).unwrap_or_default(),
                room_uid: obfuscate(m.room_id).unwrap_or_default(),
                sender_uid: obfuscate(m.sender_id).unwrap_or_default(),
                text: m.message,
                read_status: m.is_read,
                sent_at: m.created_at.to_rfc3339(),
            });
        }

        Ok(result)
    }

    async fn get_user_rooms(&self, user_id: i64) -> Result<Vec<ChatRoomRes>, ChatServiceError> {
        use crate::db::entity::{chat_participant, chat_room};
        use crate::utils::security::obfuscate;
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        let participants = chat_participant::Entity::find()
            .filter(chat_participant::Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(|e| ChatServiceError::Internal(e.to_string()))?;

        let mut rooms_res = Vec::new();
        for p in participants {
            let room = chat_room::Entity::find_by_id(p.room_id)
                .one(&self.db)
                .await
                .map_err(|e| ChatServiceError::Internal(e.to_string()))?;

            if let Some(r) = room {
                let room_participants = chat_participant::Entity::find()
                    .filter(chat_participant::Column::RoomId.eq(r.id))
                    .all(&self.db)
                    .await
                    .unwrap_or_default();

                let participant_uids = room_participants
                    .into_iter()
                    .map(|rp| obfuscate(rp.user_id).unwrap_or_default())
                    .collect();

                let product_name = if let Some(pid) = r.product_id {
                    if let Ok(Some(prod)) = crate::db::entity::product::Entity::find_by_id(pid)
                        .one(&self.db)
                        .await
                    {
                        Some(prod.title)
                    } else {
                        None
                    }
                } else {
                    None
                };

                rooms_res.push(ChatRoomRes {
                    room_uid: obfuscate(r.id).unwrap_or_default(),
                    room_kind: shared::dto::chat_dto::RoomKind::Private,
                    item_uid: r.product_id.map(|id| obfuscate(id).unwrap_or_default()),
                    product_name,
                    participant_uids,
                    created_at: r.created_at.to_rfc3339(),
                });
            }
        }
        Ok(rooms_res)
    }
}
