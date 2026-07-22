// crates/backend/src/ports/wallet_port.rs
// 목적: BCH 지갑 관련 Port 정의.
// 이 trait을 구현하는 어댑터(BchWalletAdapter)는 infra/ 계층에 위치합니다.
// Service는 이 인터페이스에만 의존하므로 BCH 라이브러리 변경 시 Service 수정 없이
// 어댑터만 교체하면 됩니다.

use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalletPortError {
    #[error("지갑 생성 실패: {0}")]
    CreateFailed(String),
    #[error("주소 유효성 검증 실패")]
    InvalidAddress,
    #[error("잔액 조회 실패: {0}")]
    BalanceFetchFailed(String),
    #[error("트랜잭션 브로드캐스트 실패: {0}")]
    BroadcastFailed(String),
    #[error("수수료 추정 실패")]
    FeesEstimationFailed,
}

/// 새로 생성된 HD 지갑 정보
#[derive(Debug)]
pub struct NewWalletInfo {
    pub bch_address: String,   // CashAddr 형식 공개 주소
    pub encrypted_privkey: Vec<u8>, // AES-GCM 암호화된 개인키
    pub encrypted_mnemonic: Vec<u8>, // AES-GCM 암호화된 니모닉
}

/// BCH 지갑 Port (인터페이스)
///
/// 구현체: infra::bch_wallet_adapter::BchWalletAdapter
#[async_trait]
pub trait BchWalletPort: Send + Sync {
    /// 새 HD 지갑(BIP39 니모닉) 생성 및 개인키 암호화 반환
    async fn create_wallet(&self, user_id: i64) -> Result<NewWalletInfo, WalletPortError>;

    /// 외부 BCH 주소 유효성 검증 (CashAddr 형식)
    async fn validate_address(&self, address: &str) -> Result<bool, WalletPortError>;

    /// 네트워크 수수료 추정 (출금 전 사용자에게 안내)
    async fn estimate_fee(&self, amount_satoshi: u64) -> Result<u64, WalletPortError>;

    /// BCH 트랜잭션 생성 및 브로드캐스트
    /// encrypted_privkey: DB에서 복호화된 원본 개인키 바이트
    async fn broadcast_transaction(
        &self,
        from_address: &str,
        to_address: &str,
        amount_satoshi: u64,
        fee_satoshi: u64,
        encrypted_privkey: &[u8],
    ) -> Result<String, WalletPortError>; // tx_hash 반환
}
