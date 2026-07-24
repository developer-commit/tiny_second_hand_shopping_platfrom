use std::sync::Arc;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
use rust_decimal::Decimal;
use shared::dto::escrow_dto::InitiateEscrowReq;
use crate::service::escrow_service::EscrowService;
use crate::ports::wallet_port::MockEvmWalletPort;
use crate::ports::notification_port::MockNotificationPort;
use crate::service::traits::EscrowServiceTrait;

#[tokio::test]
async fn test_initiate_escrow_success() {
    // 1. Mock DB (Need to mock product lookup, wallet lookup, wallet update, tx insert, escrow insert, product update)
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([
            vec![crate::db::entity::product::Model {
                id: 1,
                seller_id: 2,
                title: "Test Item".to_string(),
                description: "Test".to_string(),
                price: Decimal::new(100, 0),
                currency: "BCH".to_string(),
                category: "Test".to_string(),
                status: "on_sale".to_string(),
                view_count: 0,
                created_at: chrono::Utc::now().into(),
                updated_at: chrono::Utc::now().into(),
            }]
        ])
        .append_query_results([
            vec![crate::db::entity::wallet::Model {
                id: 1,
                user_id: 3,
                eth_address: "0xqq1".to_string(),
                created_at: chrono::Utc::now().into(),
                updated_at: chrono::Utc::now().into(),
            }]
        ])

        .append_query_results([
            vec![crate::db::entity::wallet_transaction::Model { // tx insert returns model
                id: 1,
                wallet_id: 1,
                tx_type: "escrow_lock".to_string(),
                amount: Decimal::new(100, 0),
                network_fee: Decimal::new(0, 0),
                tx_hash: None,
                status: "confirmed".to_string(),
                created_at: chrono::Utc::now().into(),
            }]
        ])
        .append_query_results([
            vec![crate::db::entity::escrow_trade::Model { // escrow insert returns model
                id: 1,
                product_id: 1,
                buyer_id: 3,
                seller_id: 2,
                amount: Decimal::new(100, 0),
                currency: "BCH".to_string(),
                platform_fee: Decimal::new(0, 0),
                status: "deposited".to_string(),
                auto_confirm_at: None,
                dispute_reason: None,
                blockchain_tx_hash: None,
                contract_trade_id: None,
                created_at: chrono::Utc::now().into(),
                updated_at: chrono::Utc::now().into(),
            }]
        ])
        .append_query_results([
            vec![crate::db::entity::product::Model { // product update returns model
                id: 1,
                seller_id: 2,
                title: "Test Item".to_string(),
                description: "Test".to_string(),
                price: Decimal::new(100, 0),
                currency: "BCH".to_string(),
                category: "Test".to_string(),
                status: "reserved".to_string(),
                view_count: 0,
                created_at: chrono::Utc::now().into(),
                updated_at: chrono::Utc::now().into(),
            }]
        ])
        .into_connection();

    // 2. Mock NotificationPort
    let mut mock_notification = MockNotificationPort::new();
    mock_notification.expect_send()
        .times(2)
        .returning(|_| Ok(()));

    // 3. Init Service
    let escrow_service = EscrowService::new(db, Arc::new(mock_notification));

    use crate::utils::security::obfuscate;
    let req = InitiateEscrowReq {
        item_uid: obfuscate(1).unwrap(),
    };

    // 5. Run test
    let result = escrow_service.initiate(3, req).await;
    assert!(result.is_ok());
}
