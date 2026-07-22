// crates/backend/src/ports/bch_network_port.rs
// 목적: BCH 블록체인 네트워크 조회 Port.
// reqwest 등의 HTTP 클라이언트 의존성을 Service에서 분리합니다.

use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BchNetworkError {
    #[error("네트워크 요청 실패: {0}")]
    RequestFailed(String),
    #[error("트랜잭션을 찾을 수 없음: {0}")]
    TxNotFound(String),
    #[error("블록체인 동기화 오류")]
    SyncError,
}

/// BCH 블록체인 네트워크 조회 Port
///
/// 구현체: infra::bch_network_adapter::BchNetworkAdapter
#[async_trait]
pub trait BchNetworkPort: Send + Sync {
    /// 특정 주소의 현재 BCH 잔액(satoshi) 조회
    async fn get_balance(&self, address: &str) -> Result<u64, BchNetworkError>;

    /// tx_hash로 트랜잭션 확인 상태 조회 (이중지불 방지)
    async fn get_tx_confirmations(&self, tx_hash: &str) -> Result<u32, BchNetworkError>;

    /// 에스크로 입금 감지 — 플랫폼 지갑에 특정 금액 입금 여부 확인
    async fn check_deposit(
        &self,
        platform_address: &str,
        expected_amount_satoshi: u64,
    ) -> Result<Option<String>, BchNetworkError>; // 확인된 tx_hash 반환
}
