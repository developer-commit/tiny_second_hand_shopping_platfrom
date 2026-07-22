// crates/backend/src/service/escrow_service.rs
// 목적: 에스크로 안전결제 흐름 비즈니스 로직.
//
// [보안 흐름]
// 1. 예치: 구매자 잔액 잠금 → 상품 상태 '예약중' 전환 → auto_confirm_at 설정(+3일)
// 2. 수령 승인: 판매자 정산 + platform_fee 차감 → 상품 '판매완료' → 알림 발송
// 3. 수령 거부: 운영진 중재 대기 (dispute 상태)
// 4. 자동 확정: cron에서 auto_confirm_at 초과 시 이 서비스의 confirm()을 자동 호출

use std::sync::Arc;
use sea_orm::DatabaseConnection;
use shared::dto::escrow_dto::{DisputeEscrowReq, InitiateEscrowReq, SafeTradeStatusRes};
use thiserror::Error;
use crate::ports::notification_port::NotificationPort;

#[derive(Debug, Error)]
pub enum EscrowServiceError {
    #[error("상품을 찾을 수 없습니다.")]
    ProductNotFound,
    #[error("거래를 찾을 수 없습니다.")]
    TradeNotFound,
    #[error("잔액이 부족합니다.")]
    InsufficientBalance,
    #[error("이미 진행 중인 에스크로 거래가 있습니다.")]
    DuplicateTrade,
    #[error("수령 승인 권한 없음 — 구매자만 가능")]
    Forbidden,
    #[error("유효하지 않은 에스크로 상태: {0}")]
    InvalidState(String),
    #[error("내부 서버 오류: {0}")]
    Internal(String),
}

pub struct EscrowService {
    db: DatabaseConnection,
    notification_port: Arc<dyn NotificationPort>,
}

impl EscrowService {
    pub fn new(
        db: DatabaseConnection,
        notification_port: Arc<dyn NotificationPort>,
    ) -> Self {
        EscrowService { db, notification_port }
    }

    /// 에스크로 예치 시작 (POST /escrow)
    pub async fn initiate(
        &self,
        buyer_id: i64,
        req: InitiateEscrowReq,
    ) -> Result<SafeTradeStatusRes, EscrowServiceError> {
        todo!("상품 조회 → 잔액 잠금 → escrow_trades INSERT → products status='reserved' → notification 발송")
    }

    /// 구매자 수령 승인 (POST /escrow/{trade_uid}/confirm)
    pub async fn confirm(
        &self,
        buyer_id: i64,
        trade_id: i64,
    ) -> Result<(), EscrowServiceError> {
        todo!("거래 조회 → 구매자 검증 → platform_fee 차감 → 판매자 정산 → status='settled' → products status='sold' → review 작성 알림")
    }

    /// 구매자 수령 거부 (POST /escrow/{trade_uid}/dispute)
    pub async fn dispute(
        &self,
        buyer_id: i64,
        trade_id: i64,
        req: DisputeEscrowReq,
    ) -> Result<(), EscrowServiceError> {
        todo!("거래 조회 → status='disputed' → dispute_reason 저장 → 관리자 알림")
    }

    /// 자동 확정 크론 작업에서 호출 (3일 타임아웃)
    pub async fn auto_confirm_expired_trades(&self) -> Result<(), EscrowServiceError> {
        todo!("escrow_trades SELECT WHERE status='deposited' AND auto_confirm_at < NOW() → confirm() 일괄 처리")
    }
}
