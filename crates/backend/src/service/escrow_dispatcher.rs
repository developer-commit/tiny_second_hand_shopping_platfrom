// crates/backend/src/service/escrow_dispatcher.rs
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait};
use std::sync::Arc;
use crate::service::traits::{EscrowServiceTrait, EscrowServiceError};
use shared::dto::escrow_dto::{InitiateEscrowReq, SafeTradeStatusRes, DisputeEscrowReq};
use crate::utils::security::deobfuscate;
use crate::db::entity::{product, escrow_trade};

pub struct EscrowDispatcherService {
    db: DatabaseConnection,
    bch_service: Arc<dyn EscrowServiceTrait>,
    eth_service: Arc<dyn EscrowServiceTrait>,
}

impl EscrowDispatcherService {
    pub fn new(
        db: DatabaseConnection,
        bch_service: Arc<dyn EscrowServiceTrait>,
        eth_service: Arc<dyn EscrowServiceTrait>,
    ) -> Self {
        Self {
            db,
            bch_service,
            eth_service,
        }
    }
}

#[async_trait]
impl EscrowServiceTrait for EscrowDispatcherService {
    async fn initiate(
        &self,
        buyer_id: i64,
        req: InitiateEscrowReq,
    ) -> Result<SafeTradeStatusRes, EscrowServiceError> {
        let product_id = deobfuscate(&req.item_uid).map_err(|_| EscrowServiceError::ProductNotFound)?;
        
        let product = product::Entity::find_by_id(product_id)
            .one(&self.db)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::ProductNotFound)?;

        match product.currency.as_str() {
            "ETH" => self.eth_service.initiate(buyer_id, req).await,
            _ => self.bch_service.initiate(buyer_id, req).await,
        }
    }

    async fn deposit(&self, buyer_id: i64, trade_id: i64) -> Result<(), EscrowServiceError> {
        self.eth_service.deposit(buyer_id, trade_id).await
    }

    async fn confirm(
        &self,
        buyer_id: i64,
        trade_id: i64,
    ) -> Result<(), EscrowServiceError> {
        let trade = escrow_trade::Entity::find_by_id(trade_id)
            .one(&self.db)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::TradeNotFound)?;

        match trade.currency.as_str() {
            "ETH" => self.eth_service.confirm(buyer_id, trade_id).await,
            _ => self.bch_service.confirm(buyer_id, trade_id).await,
        }
    }

    async fn dispute(
        &self,
        buyer_id: i64,
        trade_id: i64,
        req: DisputeEscrowReq,
    ) -> Result<(), EscrowServiceError> {
        let trade = escrow_trade::Entity::find_by_id(trade_id)
            .one(&self.db)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::TradeNotFound)?;

        match trade.currency.as_str() {
            "ETH" => self.eth_service.dispute(buyer_id, trade_id, req).await,
            _ => self.bch_service.dispute(buyer_id, trade_id, req).await,
        }
    }

    async fn get_trade(
        &self,
        user_id: i64,
        trade_id: i64,
    ) -> Result<SafeTradeStatusRes, EscrowServiceError> {
        let trade = escrow_trade::Entity::find_by_id(trade_id)
            .one(&self.db)
            .await
            .map_err(|e| EscrowServiceError::Internal(e.to_string()))?
            .ok_or(EscrowServiceError::TradeNotFound)?;

        match trade.currency.as_str() {
            "ETH" => self.eth_service.get_trade(user_id, trade_id).await,
            _ => self.bch_service.get_trade(user_id, trade_id).await,
        }
    }

    async fn auto_confirm_expired_trades(&self) -> Result<(), EscrowServiceError> {
        // Here we just call both services sequentially
        let bch_result = self.bch_service.auto_confirm_expired_trades().await;
        let eth_result = self.eth_service.auto_confirm_expired_trades().await;

        bch_result?;
        eth_result?;

        Ok(())
    }
}
