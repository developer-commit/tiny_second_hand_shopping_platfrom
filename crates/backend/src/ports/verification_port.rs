// crates/backend/src/ports/verification_port.rs
use async_trait::async_trait;
use thiserror::Error;
#[cfg(test)]
use mockall::automock;

#[derive(Debug, Error)]
pub enum VerificationPortError {
    #[error("API 호출 실패: {0}")]
    ApiError(String),
    #[error("이메일 전송 실패: {0}")]
    SendError(String),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait VerificationPort: Send + Sync {
    /// 대상(이메일 등)에게 인증 코드를 전송합니다.
    async fn send_code(&self, target: &str) -> Result<(), VerificationPortError>;
    
    /// 대상의 인증 코드가 올바른지 검증합니다.
    async fn verify_code(&self, target: &str, code: &str) -> Result<bool, VerificationPortError>;
}