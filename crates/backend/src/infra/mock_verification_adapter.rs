// crates/backend/src/infra/mock_verification_adapter.rs
use crate::ports::verification_port::{VerificationPort, VerificationPortError};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

struct AuthCode {
    code: String,
    time: DateTime<Utc>,
}

impl AuthCode {
    // &String 대신 &str을 매개변수로 사용
    fn is_verified(&self, code: &str) -> bool {
        let expiration_limit = Utc::now() - Duration::minutes(5);
        self.time > expiration_limit && self.code == code
    }
}

#[derive(Clone)]
pub struct MockVerificationAdapter {
    store: Arc<RwLock<HashMap<String, AuthCode>>>,
}

impl MockVerificationAdapter {
    pub fn new() -> Self {
        MockVerificationAdapter {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl VerificationPort for MockVerificationAdapter {
    async fn send_code(&self, target: &str) -> Result<(), VerificationPortError> {
        // 랜덤 6자리 숫자 생성 (테스트 시 고정값이 필요하다면 이 부분만 조절하세요)
        let code: String = "000000".to_string();

        let authcode = AuthCode {
            code,
            time: Utc::now(),
        };

        let mut store = self.store.write().await;
        store.insert(target.to_string(), authcode);

        Ok(())
    }

    async fn verify_code(&self, target: &str, code: &str) -> Result<bool, VerificationPortError> {
        let mut store = self.store.write().await;

        if let Some(authcode) = store.get(target) {
            if authcode.is_verified(code) {
                store.remove(target); // 검증 성공 시 삭제
                return Ok(true);
            }
        }

        Ok(false)
    }
}
