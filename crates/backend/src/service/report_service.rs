// crates/backend/src/service/report_service.rs
// 목적: 불량 상품/유저 신고 및 자동 제재 비즈니스 로직.
//
// [보안 로직]
// - 1유저 1상품 1회 신고 제한은 DB UNIQUE 제약(reports.reporter_id + product_id)으로 강제
// - 누적 신고 횟수 초과 시 상품 자동 차단 / 유저 계정 dormant 전환

use crate::service::traits::ReportServiceTrait;
use async_trait::async_trait;
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
}

#[async_trait]
impl ReportServiceTrait for ReportService {
    /// 1. report 신고 기록 생성 (동일 유저-상품 중복 방지)
    async fn submit_report(
        &self,
        reporter_id: i64,
        product_id: i64,
        req: SubmitReportReq,
    ) -> Result<ReportAckRes, ReportServiceError> {
        use crate::db::entity::{product, report};
        use crate::utils::security::obfuscate;
        use sea_orm::{
            ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait,
        };

        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| ReportServiceError::Internal(e.to_string()))?;

        let prod = product::Entity::find_by_id(product_id)
            .one(&txn)
            .await
            .map_err(|e| ReportServiceError::Internal(e.to_string()))?
            .ok_or(ReportServiceError::ProductNotFound)?;

        if prod.seller_id == reporter_id {
            return Err(ReportServiceError::SelfReport);
        }

        let existing = report::Entity::find()
            .filter(report::Column::ReporterId.eq(reporter_id))
            .filter(report::Column::ProductId.eq(product_id))
            .one(&txn)
            .await
            .map_err(|e| ReportServiceError::Internal(e.to_string()))?;

        if existing.is_some() {
            return Err(ReportServiceError::AlreadyReported);
        }

        let r = report::ActiveModel {
            reporter_id: Set(reporter_id),
            product_id: Set(product_id),
            reason: Set(req.cause.clone()),
            status: Set("pending".to_string()),
            created_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };

        let inserted = r
            .insert(&txn)
            .await
            .map_err(|e| ReportServiceError::Internal(e.to_string()))?;

        txn.commit()
            .await
            .map_err(|e| ReportServiceError::Internal(e.to_string()))?;

        // Apply auto moderation
        if let Err(e) = self.apply_auto_moderation(product_id, prod.seller_id).await {
            eprintln!("Auto moderation failed: {}", e);
        }

        Ok(ReportAckRes {
            report_uid: obfuscate(inserted.id).unwrap_or_default(),
            status: shared::dto::report_dto::ReportStatus::Pending,
            submitted_at: inserted.created_at.to_rfc3339(),
        })
    }
}

impl ReportService {
    /// 임계값 초과 시 자동 제재 처리 (내부 호출)
    async fn apply_auto_moderation(
        &self,
        product_id: i64,
        seller_id: i64,
    ) -> Result<(), ReportServiceError> {
        use crate::db::entity::{product, report, user};
        use sea_orm::{
            ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
        };

        let report_count = report::Entity::find()
            .filter(report::Column::ProductId.eq(product_id))
            .count(&self.db)
            .await
            .map_err(|e| ReportServiceError::Internal(e.to_string()))?;

        if report_count >= PRODUCT_AUTO_BLOCK_THRESHOLD as u64 {
            let prod = product::Entity::find_by_id(product_id)
                .one(&self.db)
                .await
                .map_err(|e| ReportServiceError::Internal(e.to_string()))?;

            if let Some(p) = prod {
                let mut p: product::ActiveModel = p.into();
                p.status = Set("blocked".to_string());
                p.update(&self.db)
                    .await
                    .map_err(|e| ReportServiceError::Internal(e.to_string()))?;
            }

            // Check seller's total reports across all their products
            let all_seller_products = product::Entity::find()
                .filter(product::Column::SellerId.eq(seller_id))
                .all(&self.db)
                .await
                .map_err(|e| ReportServiceError::Internal(e.to_string()))?;

            let product_ids: Vec<i64> = all_seller_products.into_iter().map(|p| p.id).collect();

            let total_seller_reports = report::Entity::find()
                .filter(report::Column::ProductId.is_in(product_ids))
                .count(&self.db)
                .await
                .map_err(|e| ReportServiceError::Internal(e.to_string()))?;

            if total_seller_reports >= USER_DORMANT_THRESHOLD as u64 {
                let u = user::Entity::find_by_id(seller_id)
                    .one(&self.db)
                    .await
                    .map_err(|e| ReportServiceError::Internal(e.to_string()))?;

                if let Some(u) = u {
                    let mut u: user::ActiveModel = u.into();
                    u.status = Set("dormant".to_string());
                    u.update(&self.db)
                        .await
                        .map_err(|e| ReportServiceError::Internal(e.to_string()))?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::entity::{product, report, user};
    use chrono::Utc;
    use rust_decimal::Decimal;
    use sea_orm::{DatabaseBackend, MockDatabase, Value};
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn test_submit_report_success() {
        let now = Utc::now().into();
        let prod_model = product::Model {
            id: 1,
            seller_id: 200,
            title: "Test".to_string(),
            description: "Test".to_string(),
            price: Decimal::new(100, 0),
            currency: "BCH".to_string(),
            category: "Elec".to_string(),
            status: "on_sale".to_string(),
            view_count: 0,
            created_at: now,
            updated_at: now,
        };

        let report_model = report::Model {
            id: 1,
            reporter_id: 100,
            product_id: 1,
            reason: "spam".to_string(),
            status: "pending".to_string(),
            created_at: now,
        };

        let mut count_map = BTreeMap::new();
        count_map.insert("num_items".to_string(), Value::BigInt(Some(1))); // Not hitting threshold

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![prod_model.clone()]])
            .append_query_results([vec![] as Vec<report::Model>])
            .append_query_results([vec![report_model.clone()]])
            .append_query_results([vec![count_map]])
            .into_connection();

        let service = ReportService::new(db);
        let req = SubmitReportReq {
            cause: "spam".to_string(),
        };
        let res = service.submit_report(100, 1, req).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_submit_report_self_report() {
        let now = Utc::now().into();
        let prod_model = product::Model {
            id: 1,
            seller_id: 100, // Same as reporter
            title: "Test".to_string(),
            description: "Test".to_string(),
            price: Decimal::new(100, 0),
            currency: "BCH".to_string(),
            category: "Elec".to_string(),
            status: "on_sale".to_string(),
            view_count: 0,
            created_at: now,
            updated_at: now,
        };

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![prod_model]])
            .into_connection();

        let service = ReportService::new(db);
        let req = SubmitReportReq {
            cause: "spam".to_string(),
        };
        let res = service.submit_report(100, 1, req).await;

        assert!(matches!(res, Err(ReportServiceError::SelfReport)));
    }
}
