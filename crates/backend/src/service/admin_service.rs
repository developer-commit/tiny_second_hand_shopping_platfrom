// crates/backend/src/service/admin_service.rs
// 목적: 관리자 전용 플랫폼 통계 조회 및 분쟁 강제 정산 비즈니스 로직.
//
// [보안 제약]
// - 이 서비스의 모든 메서드는 rbac::require_admin 미들웨어로 보호된 라우터에서만 호출됩니다.
// - 강제 정산은 반드시 감사 로그(audit log)에 reason과 관리자 ID를 기록해야 합니다.

use std::sync::Arc;
use sea_orm::DatabaseConnection;
use shared::dto::admin_dto::{ForceSettleReq, ForceSettleTarget, PlatformStatsRes, BanUserReq, HideProductReq, AdminReportListRes, AdminReportSummary};
use thiserror::Error;
use async_trait::async_trait;
use crate::service::traits::{AdminServiceTrait, WalletServiceTrait};

#[derive(Debug, Error, PartialEq)]
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
    wallet_service: Arc<dyn WalletServiceTrait>,
}

impl AdminService {
    pub fn new(db: DatabaseConnection, wallet_service: Arc<dyn WalletServiceTrait>) -> Self {
        AdminService { db, wallet_service }
    }
}

#[async_trait]
impl AdminServiceTrait for AdminService {
    /// 플랫폼 통계 조회 (GET /admin/stats)
    async fn get_platform_stats(&self) -> Result<PlatformStatsRes, AdminServiceError> {
        use crate::db::entity::{user, product, escrow_trade, report, platform_stats};
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, PaginatorTrait};
        
        let users_count = user::Entity::find().count(&self.db).await.map_err(|e| AdminServiceError::Internal(e.to_string()))?;
        let products_count = product::Entity::find().filter(product::Column::Status.eq("on_sale")).count(&self.db).await.map_err(|e| AdminServiceError::Internal(e.to_string()))?;
        let escrows_count = escrow_trade::Entity::find().filter(escrow_trade::Column::Status.eq("deposited")).count(&self.db).await.map_err(|e| AdminServiceError::Internal(e.to_string()))?;
        let disputes_count = escrow_trade::Entity::find().filter(escrow_trade::Column::Status.eq("disputed")).count(&self.db).await.map_err(|e| AdminServiceError::Internal(e.to_string()))?;
        let reports_count = report::Entity::find().filter(report::Column::Status.eq("pending")).count(&self.db).await.map_err(|e| AdminServiceError::Internal(e.to_string()))?;
        
        let stats = platform_stats::Entity::find().one(&self.db).await.map_err(|e| AdminServiceError::Internal(e.to_string()))?;
        let (total_accumulated_bch, last_calculated_at) = match stats {
            Some(s) => {
                let accumulated = s.total_fee_collected
                    .try_into()
                    .map_err(|_| AdminServiceError::Internal("Fee type conversion failed".into()))?;
                (accumulated, s.updated_at.to_rfc3339())
            },
            None => (0.0, chrono::Utc::now().to_rfc3339()),
        };

        Ok(PlatformStatsRes {
            total_accumulated_bch,
            total_users: users_count,
            active_listings: products_count,
            active_escrows: escrows_count,
            pending_disputes: disputes_count,
            pending_reports: reports_count,
            last_calculated_at,
        })
    }

    /// 분쟁 강제 정산 (POST /admin/escrow/{trade_uid}/force-settle)
    /// 감사 로그: admin_id, trade_id, direction, reason, timestamp 기록
    async fn force_settle(
        &self,
        admin_id: i64,
        req: ForceSettleReq,
    ) -> Result<(), AdminServiceError> {
        use crate::db::entity::escrow_trade;
        use sea_orm::{EntityTrait, ActiveModelTrait, Set, TransactionTrait};
        use crate::utils::security::deobfuscate;

        let trade_id = deobfuscate(&req.trade_uid).map_err(|_| AdminServiceError::TradeNotFound)?;
        
        let txn = self.db.begin().await.map_err(|e| AdminServiceError::Internal(e.to_string()))?;

        let trade = escrow_trade::Entity::find_by_id(trade_id)
            .one(&txn)
            .await
            .map_err(|e| AdminServiceError::Internal(e.to_string()))?
            .ok_or(AdminServiceError::TradeNotFound)?;

        if trade.status != "disputed" {
            return Err(AdminServiceError::AlreadySettled);
        }

        let new_status = match req.settle_to {
            ForceSettleTarget::Buyer => "refunded",
            ForceSettleTarget::Seller => "settled",
        };

        let mut active_trade: escrow_trade::ActiveModel = trade.clone().into();
        active_trade.status = Set(new_status.to_string());
        active_trade.updated_at = Set(chrono::Utc::now().into());
        active_trade.update(&txn).await.map_err(|e| AdminServiceError::Internal(e.to_string()))?;

        if req.settle_to == ForceSettleTarget::Seller {
            // Note: In an EVM architecture, forcing a settle to the seller requires dispatching an on-chain transaction.
            // Currently, this only updates DB escrow state. 
            tracing::warn!("Admin forced settle to seller, but on-chain payout is not dispatched.");
        }

        txn.commit().await.map_err(|e| AdminServiceError::Internal(e.to_string()))?;

        tracing::info!(
            admin_id = admin_id,
            trade_id = trade_id,
            direction = ?req.settle_to,
            reason = %req.reason,
            "AUDIT_LOG: Force settle executed"
        );

        Ok(())
    }

    /// 사용자 밴 (POST /admin/users/{user_uid}/ban)
    async fn ban_user(&self, admin_id: i64, user_id: i64, req: BanUserReq) -> Result<(), AdminServiceError> {
        use crate::db::entity::user;
        use sea_orm::{EntityTrait, ActiveModelTrait, Set};

        let user_model = user::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| AdminServiceError::Internal(e.to_string()))?
            .ok_or_else(|| AdminServiceError::Internal("User not found".to_string()))?;

        let mut active_user: user::ActiveModel = user_model.into();
        active_user.status = Set("banned".to_string());
        active_user.updated_at = Set(chrono::Utc::now().into());
        active_user.update(&self.db).await.map_err(|e| AdminServiceError::Internal(e.to_string()))?;

        tracing::info!(admin_id = admin_id, user_id = user_id, reason = %req.reason, "AUDIT_LOG: User banned");

        Ok(())
    }

    /// 상품 숨김 (POST /admin/products/{item_uid}/hide)
    async fn hide_product(&self, admin_id: i64, product_id: i64, req: HideProductReq) -> Result<(), AdminServiceError> {
        use crate::db::entity::product;
        use sea_orm::{EntityTrait, ActiveModelTrait, Set};

        let product_model = product::Entity::find_by_id(product_id)
            .one(&self.db)
            .await
            .map_err(|e| AdminServiceError::Internal(e.to_string()))?
            .ok_or_else(|| AdminServiceError::Internal("Product not found".to_string()))?;

        let mut active_product: product::ActiveModel = product_model.into();
        active_product.status = Set("hidden".to_string());
        active_product.updated_at = Set(chrono::Utc::now().into());
        active_product.update(&self.db).await.map_err(|e| AdminServiceError::Internal(e.to_string()))?;

        tracing::info!(admin_id = admin_id, product_id = product_id, reason = %req.reason, "AUDIT_LOG: Product hidden");

        Ok(())
    }

    /// 신고 목록 조회 (GET /admin/reports)
    async fn list_reports(&self) -> Result<AdminReportListRes, AdminServiceError> {
        use crate::db::entity::report;
        use sea_orm::EntityTrait;
        use crate::utils::security::obfuscate;

        let reports = report::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| AdminServiceError::Internal(e.to_string()))?;

        let summaries = reports.into_iter().map(|r| AdminReportSummary {
            report_uid: obfuscate(r.id).unwrap_or_default(),
            target_item_uid: obfuscate(r.product_id).unwrap_or_default(),
            reporter_uid: obfuscate(r.reporter_id).unwrap_or_default(),
            cause: r.reason,
            status: r.status,
            submitted_at: r.created_at.to_rfc3339(),
        }).collect();

        Ok(AdminReportListRes { reports: summaries })
    }
}


