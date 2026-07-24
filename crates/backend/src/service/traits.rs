// crates/backend/src/service/traits.rs
use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;

use shared::dto::{
    admin_dto::{ForceSettleReq, PlatformStatsRes, BanUserReq, HideProductReq, AdminReportListRes}, chat_dto::{ChatMessagePayload, ChatRoomRes, CreateChatRoomReq, SendMessageReq}, escrow_dto::{DisputeEscrowReq, InitiateEscrowReq, SafeTradeStatusRes}, noti_dto::NotificationRes, product_dto::{CreateItemReq, ItemDetailRes, ItemSummaryRes, ProductSearchQuery, UpdateItemReq, UpdateItemStateReq}, report_dto::{ReportAckRes, SubmitReportReq}, review_dto::{ReviewRes, SubmitReviewReq}, transaction_dto::{TxHistoryItemRes, WalletStateRes, EthWithdrawReq}, user_dto::{AuthTokenRes, Enable2FaReq, LoginReq, LoginResponse, SendCodeReq, SignUpReq, TwoFaSetupRes, UpdateProfileReq, UserProfileRes},
};

//errors
use thiserror::Error;
#[derive(Debug, Error)]
pub enum EscrowServiceError {
    #[error("상품을 찾을 수 없습니다.")]
    ProductNotFound,
    #[error("에스크로가 아직 시작되지 않았습니다.")]
    EscrowNotInitiated,
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
    #[error("유효하지 않은 지갑 주소입니다.")]
    InvalidWalletAddress,
    #[error("스마트 컨트랙트 실행 실패: {0}")]
    ContractRevert(String),
    #[error("RPC 네트워크 오류: {0}")]
    RpcError(String),
    #[error("내부 서버 오류: {0}")]
    Internal(String),
}

use super::{
    admin_service::AdminServiceError,
    auth_service::AuthServiceError,
    chat_service::ChatServiceError,
    notification_service::NotificationServiceError,
    product_service::ProductServiceError,
    report_service::ReportServiceError,
    review_service::ReviewServiceError,
    user_service::UserServiceError,
    wallet_service::WalletServiceError,
};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait AdminServiceTrait: Send + Sync {
    async fn get_platform_stats(&self) -> Result<PlatformStatsRes, AdminServiceError>;
    async fn force_settle(&self, admin_id: i64, req: ForceSettleReq) -> Result<(), AdminServiceError>;
    async fn ban_user(&self, admin_id: i64, user_id: i64, req: BanUserReq) -> Result<(), AdminServiceError>;
    async fn hide_product(&self, admin_id: i64, product_id: i64, req: HideProductReq) -> Result<(), AdminServiceError>;
    async fn list_reports(&self) -> Result<AdminReportListRes, AdminServiceError>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait AuthServiceTrait: Send + Sync {
    async fn send_code(&self, req: SendCodeReq) -> Result<(), AuthServiceError>;
    async fn sign_up(&self, req: SignUpReq) -> Result<(), AuthServiceError>;
    async fn login(&self, req: LoginReq) -> Result<LoginResponse, AuthServiceError>;
    async fn setup_2fa(&self, user_id: i64) -> Result<TwoFaSetupRes, AuthServiceError>;
    async fn enable_2fa(&self, user_id: i64, req: Enable2FaReq) -> Result<(), AuthServiceError>;
    async fn verify_otp(&self, user_id: i64, otp_token: &str) -> Result<(), AuthServiceError>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ChatServiceTrait: Send + Sync {
    async fn get_or_create_room(&self, user_id: i64, req: CreateChatRoomReq) -> Result<ChatRoomRes, ChatServiceError>;
    async fn send_message(&self, sender_id: i64, req: SendMessageReq) -> Result<ChatMessagePayload, ChatServiceError>;
    async fn get_history(&self, user_id: i64, room_id: i64, before_msg_id: Option<i64>) -> Result<Vec<ChatMessagePayload>, ChatServiceError>;
    async fn get_user_rooms(&self, user_id: i64) -> Result<Vec<ChatRoomRes>, ChatServiceError>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait EscrowServiceTrait: Send + Sync {
    async fn initiate(&self, buyer_id: i64, req: InitiateEscrowReq) -> Result<SafeTradeStatusRes, EscrowServiceError>;
    async fn deposit(&self, buyer_id: i64, trade_id: i64) -> Result<(), EscrowServiceError>;
    async fn confirm(&self, buyer_id: i64, trade_id: i64) -> Result<(), EscrowServiceError>;
    async fn dispute(&self, buyer_id: i64, trade_id: i64, req: DisputeEscrowReq) -> Result<(), EscrowServiceError>;
    async fn get_trade(&self, user_id: i64, trade_id: i64) -> Result<SafeTradeStatusRes, EscrowServiceError>;
    async fn auto_confirm_expired_trades(&self) -> Result<(), EscrowServiceError>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait NotificationServiceTrait: Send + Sync {
    async fn get_notifications(&self, user_id: i64) -> Result<Vec<NotificationRes>, NotificationServiceError>;
    async fn mark_as_read(&self, user_id: i64, noti_id: i64) -> Result<(), NotificationServiceError>;
    async fn send_notification(&self, user_id: i64, noti_type: &str, reference_id: Option<i64>, message: &str) -> Result<(), NotificationServiceError>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ProductServiceTrait: Send + Sync {
    async fn list_products(&self, query: ProductSearchQuery) -> Result<Vec<ItemSummaryRes>, ProductServiceError>;
    async fn get_product(&self, product_id: i64) -> Result<ItemDetailRes, ProductServiceError>;
    async fn create_product(&self, seller_id: i64, req: CreateItemReq) -> Result<ItemDetailRes, ProductServiceError>;
    async fn update_product(&self, seller_id: i64, product_id: i64, req: UpdateItemReq) -> Result<ItemDetailRes, ProductServiceError>;
    async fn update_product_state(&self, seller_id: i64, product_id: i64, req: UpdateItemStateReq) -> Result<(), ProductServiceError>;
    async fn delete_product(&self, seller_id: i64, product_id: i64) -> Result<(), ProductServiceError>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ReportServiceTrait: Send + Sync {
    async fn submit_report(&self, reporter_id: i64, product_id: i64, req: SubmitReportReq) -> Result<ReportAckRes, ReportServiceError>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ReviewServiceTrait: Send + Sync {
    async fn submit_review(&self, reviewer_id: i64, trade_id: i64, req: SubmitReviewReq) -> Result<ReviewRes, ReviewServiceError>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait UserServiceTrait: Send + Sync {
    async fn get_profile(&self, user_id: i64) -> Result<UserProfileRes, UserServiceError>;
    async fn update_profile(&self, user_id: i64, req: UpdateProfileReq) -> Result<UserProfileRes, UserServiceError>;
    async fn recalculate_trust_score(&self, user_id: i64) -> Result<(), UserServiceError>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait WalletServiceTrait: Send + Sync {
    async fn get_wallet_state(&self, user_id: i64) -> Result<WalletStateRes, WalletServiceError>;

    async fn get_tx_history(&self, user_id: i64) -> Result<Vec<TxHistoryItemRes>, WalletServiceError>;
    async fn eth_withdraw(&self, user_id: i64, req: EthWithdrawReq) -> Result<TxHistoryItemRes, WalletServiceError>;
}
