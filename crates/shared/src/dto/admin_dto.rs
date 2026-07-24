// crates/shared/src/dto/admin_dto.rs
// 목적: 관리자 전용 플랫폼 통계 및 운영 DTO.
// 접근 제어: 이 DTO를 사용하는 모든 핸들러는 backend의
//           rbac::require_admin 미들웨어로 보호됩니다.
// 스키마 은닉: total_fee_collected → total_accumulated_fees,
//             updated_at → last_calculated_at

use crate::types::OpaqueId;
use serde::{Deserialize, Serialize};

/// [Response] GET /admin/stats — 플랫폼 누적 통계
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformStatsRes {
    pub total_accumulated_fees: f64, // DB: total_fee_collected
    pub total_users: u64,            // users 테이블 COUNT
    pub active_listings: u64,        // products WHERE status='on_sale' COUNT
    pub active_escrows: u64,         // escrow_trades WHERE status='deposited' COUNT
    pub pending_disputes: u64,       // escrow_trades WHERE status='disputed' COUNT
    pub pending_reports: u64,        // reports WHERE status='pending' COUNT
    pub last_calculated_at: String,  // DB: updated_at
}

/// [Request] POST /admin/escrow/{trade_uid}/force-settle — 강제 정산
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForceSettleReq {
    pub trade_uid: OpaqueId,
    pub settle_to: ForceSettleTarget,
    pub reason: String, // 감사 로그용
}

/// 강제 정산 방향
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ForceSettleTarget {
    Buyer,  // 구매자에게 환불
    Seller, // 판매자에게 정산
}

/// [Response] 관리자 신고 목록
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminReportSummary {
    pub report_uid: OpaqueId,
    pub target_item_uid: OpaqueId,
    pub reporter_uid: OpaqueId,
    pub cause: String,
    pub status: String,
    pub submitted_at: String,
}

/// [Request] POST /admin/users/{user_uid}/ban — 사용자 밴
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanUserReq {
    pub reason: String,
}

/// [Request] POST /admin/products/{item_uid}/hide — 상품 숨김
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HideProductReq {
    pub reason: String,
}

/// [Response] GET /admin/reports — 신고 목록
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminReportListRes {
    pub reports: Vec<AdminReportSummary>,
}
