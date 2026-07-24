use std::sync::Arc;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
use shared::dto::user_dto::SignUpReq;
use crate::service::auth_service::AuthService;
use crate::ports::verification_port::MockVerificationPort;
use crate::ports::wallet_port::{MockEvmWalletPort, NewWalletInfo};
use crate::service::traits::AuthServiceTrait;

#[tokio::test]
async fn test_sign_up_success() {
    // 1. Mock DB
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([
            Vec::<crate::db::entity::user::Model>::new() // user lookup returns empty
        ])
        .append_query_results([
            vec![crate::db::entity::user::Model {
                id: 1,
                username: "testuser".to_string(),
                password_hash: "dummyhash".to_string(),
                email: Some("test@example.com".to_string()),
                phone: None,
                is_verified: false,
                bio: None,
                trust_score: rust_decimal::Decimal::new(50, 0),
                is_2fa_enabled: false,
                two_factor_secret: None,
                role: "user".to_string(),
                status: "active".to_string(),
                reported_count: 0,
                created_at: chrono::Utc::now().into(),
                updated_at: chrono::Utc::now().into(),
            }]
        ])
        .append_query_results([
            vec![crate::db::entity::wallet::Model {
                id: 1,
                user_id: 1,
                eth_address: "0xdummy".to_string(),
                created_at: chrono::Utc::now().into(),
                updated_at: chrono::Utc::now().into(),
            }]
        ])
        .into_connection();

    // 2. Mock VerificationPort
    let mut mock_verification = MockVerificationPort::new();
    mock_verification.expect_verify_code()
        .with(mockall::predicate::eq("test@example.com"), mockall::predicate::eq("dummy_code"))
        .times(1)
        .returning(|_, _| Ok(true));

    // 3. Mock EvmWalletPort
    let mut mock_wallet = MockEvmWalletPort::new();
    mock_wallet.expect_create_wallet()
        .times(1)
        .returning(|_| Ok(NewWalletInfo {
            eth_address: "0xdummy".to_string(),
            encrypted_privkey: vec![1, 2, 3],
            encrypted_mnemonic: vec![4, 5, 6],
        }));

    // 4. Init Service
    use crate::utils::auth::JwtSecret;
    use crate::utils::security::SecretString;
    let jwt_secret = Arc::new(JwtSecret(SecretString::new("testsecret".to_string())));
    let auth_service = AuthService::new(db, jwt_secret, Arc::new(mock_verification), Arc::new(mock_wallet));

    // 5. Run test
    let req = SignUpReq {
        account_id: "testuser".to_string(),
        secret_key: "Password123!".to_string(),
        contact_email: Some("test@example.com".to_string()),
        contact_phone: None,
        verification_code: "dummy_code".to_string(),
    };

    let result = auth_service.sign_up(req).await;
    assert!(result.is_ok());
}
