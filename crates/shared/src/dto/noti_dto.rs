// crates/shared/src/dto/noti_dto.rs
// 목적: 알림 시스템 DTO.
// 스키마 은닉: type → kind, reference_id → link_uid (난독화)

use crate::types::OpaqueId;
use serde::{Deserialize, Serialize};

/// 알림 유형 Enum — DB의 VARCHAR type 컬럼을 타입 안전하게 표현
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationKind {
    Chat,         // DB: "chat"
    EscrowUpdate, // DB: "escrow_update"
    System,       // DB: "system"
    Warning,      // DB: "warning"
    ReportResult, // DB: "report_result"
}

/// [Response] 알림 항목
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRes {
    pub noti_uid: OpaqueId,         // DB: id (난독화)
    pub kind: NotificationKind,     // DB: type
    pub link_uid: Option<OpaqueId>, // DB: reference_id (난독화)
    pub content: String,            // DB: message
    pub is_read: bool,              // DB: is_read
    pub received_at: String,        // DB: created_at
}
