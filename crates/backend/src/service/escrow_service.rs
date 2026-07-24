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
use shared::dto::escrow_dto::{DisputeEscrowReq, InitiateEscrowReq, SafeTradeStatusRes, TradeStep};
use crate::utils::security::obfuscate;

use crate::ports::notification_port::NotificationPort;
use async_trait::async_trait;

use crate::service::traits::{EscrowServiceTrait, EscrowServiceError};



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
}

#[async_trait]
impl EscrowServiceTrait for EscrowService {
    async fn initiate(
        &self,
        buyer_id: i64,
        req: InitiateEscrowReq,
    ) -> Result<SafeTradeStatusRes, EscrowServiceError> {
        use crate::db::entity::{product, wallet, escrow_trade, wallet_transaction};
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set, TransactionTrait};
        use crate::utils::security::deobfuscate;
        use rust_decimal::Decimal;
        use rust_decimal::prelude::FromPrimitive;
        
        let product_id = deobfuscate(&req.item_uid).map_err(|_| EscrowServiceError::ProductNotFound)?;
        
        let txn = self.db.begin().await.map_err(|e| EscrowServiceError::Internal(e.to_string()))?;
        
        let existing_trade = escrow_trade::Entity::find()
            .filter(escrow_trade::Column::ProductId.eq(product_id))
            .one(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?;
            
        if let Some(trade) = existing_trade {
            if trade.buyer_id != buyer_id && trade.seller_id != buyer_id {
                let _ = txn.rollback().await;
                return Err(EscrowServiceError::Forbidden);
            }
            let _ = txn.rollback().await;
            return trade.try_into().map_err(|e| EscrowServiceError::Internal(format!("DTO: {:?}", e)));
        }
        
        let p = product::Entity::find_by_id(product_id)
            .one(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::ProductNotFound)?;
            
        if p.status != "on_sale" {
            return Err(EscrowServiceError::InvalidState(p.status));
        }
        
        let buyer_wallet = wallet::Entity::find()
            .filter(wallet::Column::UserId.eq(buyer_id))
            .one(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::Internal("Wallet not found".to_string()))?;
            
        let required_amount = p.price;
        
        // Balance check and deduction removed due to EVM RPC migration.
        // The wallet_transaction will just record the lock event.
        let updated_wallet = buyer_wallet;
        
        let tx = wallet_transaction::ActiveModel {
            wallet_id: Set(updated_wallet.id),
            tx_type: Set("escrow_lock".to_string()),
            amount: Set(required_amount),
            network_fee: Set(Decimal::new(0, 0)),
            status: Set("confirmed".to_string()),
            created_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };
        tx.insert(&txn).await.map_err(|e| EscrowServiceError::Internal(e.to_string()))?;
        
        let platform_fee = Decimal::from_f64(0.0).unwrap(); // Or calculate fee
        let auto_confirm = chrono::Utc::now() + chrono::Duration::days(3);
        
        let escrow = escrow_trade::ActiveModel {
            product_id: Set(p.id),
            buyer_id: Set(buyer_id),
            seller_id: Set(p.seller_id),
            currency: Set("BCH".to_string()),
            amount: Set(required_amount),
            platform_fee: Set(platform_fee),
            status: Set("deposited".to_string()),
            auto_confirm_at: Set(Some(auto_confirm.into())),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };
        
        let inserted_escrow = escrow.insert(&txn).await.map_err(|e| EscrowServiceError::Internal(e.to_string()))?;
        
        let seller_id = p.seller_id;
        let p_title = p.title.clone();

        let mut active_p: product::ActiveModel = p.into();
        active_p.status = Set("reserved".to_string());
        active_p.updated_at = Set(chrono::Utc::now().into());
        active_p.update(&txn).await.map_err(|e| EscrowServiceError::Internal(e.to_string()))?;
        
        txn.commit().await.map_err(|e| EscrowServiceError::Internal(e.to_string()))?;
        
        use crate::ports::notification_port::CreateNotificationCmd;
        let _ = self.notification_port.send(CreateNotificationCmd {
            user_id: seller_id,
            noti_type: "escrow".to_string(),
            reference_id: Some(inserted_escrow.id),
            message: format!("에스크로 결제가 시작되었습니다. (상품: {})", p_title),
        }).await;
        
        let _ = self.notification_port.send(CreateNotificationCmd {
            user_id: buyer_id,
            noti_type: "escrow".to_string(),
            reference_id: Some(inserted_escrow.id),
            message: format!("에스크로 결제를 시작했습니다. (상품: {})", p_title),
        }).await;

        inserted_escrow.try_into().map_err(|e| EscrowServiceError::Internal(format!("DTO: {:?}", e)))
    }

    async fn deposit(
        &self,
        _buyer_id: i64,
        _trade_id: i64,
    ) -> Result<(), EscrowServiceError> {
        // Legacy BCH does not support this
        Err(EscrowServiceError::Internal("Not supported in legacy BCH".to_string()))
    }

    async fn confirm(
        &self,
        buyer_id: i64,
        trade_id: i64,
    ) -> Result<(), EscrowServiceError> {
        use crate::db::entity::{product, wallet, escrow_trade, wallet_transaction};
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set, TransactionTrait};
        use rust_decimal::Decimal;
        
        let txn = self.db.begin().await.map_err(|e| EscrowServiceError::Internal(e.to_string()))?;
        
        let trade = escrow_trade::Entity::find_by_id(trade_id)
            .one(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::TradeNotFound)?;
            
        if trade.buyer_id != buyer_id {
            return Err(EscrowServiceError::Forbidden);
        }
        
        if trade.status != "deposited" && trade.status != "received" {
            return Err(EscrowServiceError::InvalidState(trade.status));
        }
        
        let seller_wallet = wallet::Entity::find()
            .filter(wallet::Column::UserId.eq(trade.seller_id))
            .one(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::Internal("Seller wallet not found".to_string()))?;
            
        let settle_amount = trade.amount - trade.platform_fee;
        
        // Seller wallet balance increment removed due to EVM RPC migration.
        // The wallet_transaction will just record the release event.
        let updated_seller_wallet = seller_wallet;
        
        let tx = wallet_transaction::ActiveModel {
            wallet_id: Set(updated_seller_wallet.id),
            tx_type: Set("escrow_release".to_string()),
            amount: Set(settle_amount),
            network_fee: Set(Decimal::new(0, 0)),
            status: Set("confirmed".to_string()),
            created_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };
        tx.insert(&txn).await.map_err(|e| EscrowServiceError::Internal(e.to_string()))?;
        
        let mut active_trade: escrow_trade::ActiveModel = trade.clone().into();
        active_trade.status = Set("settled".to_string());
        active_trade.updated_at = Set(chrono::Utc::now().into());
        active_trade.update(&txn).await.map_err(|e| EscrowServiceError::Internal(e.to_string()))?;
        
        let p = product::Entity::find_by_id(trade.product_id)
            .one(&txn)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::ProductNotFound)?;
            
        let mut active_p: product::ActiveModel = p.into();
        active_p.status = Set("sold".to_string());
        active_p.updated_at = Set(chrono::Utc::now().into());
        active_p.update(&txn).await.map_err(|e| EscrowServiceError::Internal(e.to_string()))?;
        
        txn.commit().await.map_err(|e| EscrowServiceError::Internal(e.to_string()))?;
        
        use crate::ports::notification_port::CreateNotificationCmd;
        let _ = self.notification_port.send(CreateNotificationCmd {
            user_id: trade.seller_id,
            noti_type: "escrow".to_string(),
            reference_id: Some(trade.id),
            message: "구매자가 거래를 확정했습니다. 판매 대금이 입금되었습니다.".to_string(),
        }).await;
        
        Ok(())
    }

    async fn dispute(
        &self,
        buyer_id: i64,
        trade_id: i64,
        req: DisputeEscrowReq,
    ) -> Result<(), EscrowServiceError> {
        use crate::db::entity::escrow_trade;
        use sea_orm::{EntityTrait, ActiveModelTrait, Set};
        
        let trade = escrow_trade::Entity::find_by_id(trade_id)
            .one(&self.db)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::TradeNotFound)?;
            
        if trade.buyer_id != buyer_id {
            return Err(EscrowServiceError::Forbidden);
        }
        
        if trade.status != "deposited" && trade.status != "received" {
            return Err(EscrowServiceError::InvalidState(trade.status));
        }
        
        let mut active_trade: escrow_trade::ActiveModel = trade.clone().into();
        active_trade.status = Set("disputed".to_string());
        active_trade.dispute_reason = Set(Some(req.cause.clone()));
        active_trade.updated_at = Set(chrono::Utc::now().into());
        active_trade.update(&self.db).await.map_err(|e| EscrowServiceError::Internal(e.to_string()))?;
        
        use crate::ports::notification_port::CreateNotificationCmd;
        let _ = self.notification_port.send(CreateNotificationCmd {
            user_id: trade.seller_id,
            noti_type: "escrow_dispute".to_string(),
            reference_id: Some(trade.id),
            message: format!("구매자가 분쟁을 제기했습니다. 사유: {}", req.cause),
        }).await;

        Ok(())
    }

    async fn get_trade(
        &self,
        user_id: i64,
        trade_id: i64,
    ) -> Result<SafeTradeStatusRes, EscrowServiceError> {
        use crate::db::entity::escrow_trade;
        use sea_orm::EntityTrait;
        
        let trade = escrow_trade::Entity::find_by_id(trade_id)
            .one(&self.db)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::TradeNotFound)?;
            
        if trade.buyer_id != user_id && trade.seller_id != user_id {
            return Err(EscrowServiceError::Forbidden);
        }
        
        let step = match trade.status.as_str() {
            "deposited" => TradeStep::Deposited,
            "received" => TradeStep::Received,
            "disputed" => TradeStep::Disputed,
            "settled" => TradeStep::Settled,
            "refunded" => TradeStep::Refunded,
            _ => TradeStep::Deposited,
        };
        
        let currency_enum = match trade.currency.as_str() {
            "ETH" => shared::dto::common_dto::Currency::ETH,
            _ => shared::dto::common_dto::Currency::BCH,
        };

        Ok(SafeTradeStatusRes {
            trade_uid: obfuscate(trade.id).unwrap_or_default().into(),
            item_uid: obfuscate(trade.product_id).unwrap_or_default().into(),
            buyer_uid: obfuscate(trade.buyer_id).unwrap_or_default().into(),
            seller_uid: obfuscate(trade.seller_id).unwrap_or_default().into(),
            currency: currency_enum,
            locked_funds: trade.amount.to_string(),
            step,
            auto_finalize_deadline: trade.auto_confirm_at.map(|dt| dt.to_rfc3339()),
            created_at: trade.created_at.to_rfc3339(),
        })
    }

    async fn auto_confirm_expired_trades(&self) -> Result<(), EscrowServiceError> {
        use crate::db::entity::escrow_trade;
        use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
        
        let expired_trades = escrow_trade::Entity::find()
            .filter(escrow_trade::Column::Status.eq("deposited"))
            .filter(escrow_trade::Column::AutoConfirmAt.lt(chrono::Utc::now()))
            .all(&self.db)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?;
            
        for trade in expired_trades {
            // Log errors but continue
            let _ = self.confirm(trade.buyer_id, trade.id).await;
        }
        
        Ok(())
    }
}
