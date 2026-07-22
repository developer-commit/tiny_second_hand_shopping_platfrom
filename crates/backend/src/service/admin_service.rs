// crates/backend/src/service/admin_service.rs
// 목적: 관리자 전용 플랫폼 통계 조회 및 분쟁 강제 정산 비즈니스 로직.
//
// [보안 제약]
// - 이 서비스의 모든 메서드는 rbac::require_admin 미들웨어로 보호된 라우터에서만 호출됩니다.
// - 강제 정산은 반드시 감사 로그(audit log)에 reason과 관리자 ID를 기록해야 합니다.

use sea_orm::DatabaseConnection;
use shared::dto::admin_dto::{ForceSettleReq, ForceSettleTarget, PlatformStatsRes};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdminServiceError {
    #[error("거래를 찾을 수 없습니다.")]
    TradeNotFound,
    #[error("이미 정산된 거래입니다.")]
    AlreadySettled,
    #[error("내부 서버 오류: {0}")]
    Internal(String),
}

pub struct AdminService {
    db: DatabaseConnection,
}

impl AdminService {
    pub fn new(db: DatabaseConnection) -> Self {
        AdminService { db }
    }

    /// 플랫폼 통계 조회 (GET /admin/stats)
    pub async fn get_platform_stats(&self) -> Result<PlatformStatsRes, AdminServiceError> {
        todo!("platform_stats SELECT + users COUNT + products COUNT + escrow_trades GROUP BY status → PlatformStatsRes")
    }

    /// 분쟁 강제 정산 (POST /admin/escrow/{trade_uid}/force-settle)
    /// 감사 로그: admin_id, trade_id, direction, reason, timestamp 기록
    pub async fn force_settle(
        &self,
        admin_id: i64,
        req: ForceSettleReq,
    ) -> Result<(), AdminServiceError> {
        todo!("거래 조회 → disputed 상태 확인 → ForceSettleTarget 방향으로 정산 → 감사 로그 INSERT")
    }
}
