// crates/backend/src/service/report_service.rs
// 목적: 불량 상품/유저 신고 및 자동 제재 비즈니스 로직.
//
// [보안 로직]
// - 1유저 1상품 1회 신고 제한은 DB UNIQUE 제약(reports.reporter_id + product_id)으로 강제
// - 누적 신고 횟수 초과 시 상품 자동 차단 / 유저 계정 dormant 전환

use sea_orm::DatabaseConnection;
use shared::dto::report_dto::{ReportAckRes, SubmitReportReq};
use thiserror::Error;

/// 자동 제재 임계값
const PRODUCT_AUTO_BLOCK_THRESHOLD: i32 = 5;
const USER_DORMANT_THRESHOLD: i32 = 10;

#[derive(Debug, Error)]
pub enum ReportServiceError {
    #[error("이미 신고한 상품입니다.")]
    AlreadyReported,
    #[error("자기 자신의 상품을 신고할 수 없습니다.")]
    SelfReport,
    #[error("상품을 찾을 수 없습니다.")]
    ProductNotFound,
    #[error("내부 서버 오류: {0}")]
    Internal(String),
}

pub struct ReportService {
    db: DatabaseConnection,
}

impl ReportService {
    pub fn new(db: DatabaseConnection) -> Self {
        ReportService { db }
    }

    /// 상품 신고 접수 (POST /products/{item_uid}/reports)
    pub async fn submit_report(
        &self,
        reporter_id: i64,
        product_id: i64,
        req: SubmitReportReq,
    ) -> Result<ReportAckRes, ReportServiceError> {
        todo!("자기신고 확인 → reports INSERT(UNIQUE 위반 시 AlreadyReported) → 누적 신고 확인 → 임계값 초과 시 자동 제재")
    }

    /// 임계값 초과 시 자동 제재 처리 (내부 호출)
    async fn apply_auto_moderation(
        &self,
        product_id: i64,
        seller_id: i64,
    ) -> Result<(), ReportServiceError> {
        todo!("신고 횟수 조회 → PRODUCT_AUTO_BLOCK_THRESHOLD 초과 시 products status 차단 → USER_DORMANT_THRESHOLD 초과 시 users status='dormant'")
    }
}
