// crates/backend/src/infra/bch_wallet_adapter.rs
// 목적: BchWalletPort의 실제 구현체 (Hexagonal Architecture Adapter).
// bip39, bitcoin, cashaddr 크레이트를 사용하여 BCH 지갑을 생성/관리합니다.
// Service 계층은 이 구조체의 존재를 알지 못하고 BchWalletPort trait에만 의존합니다.

use async_trait::async_trait;
use crate::ports::wallet_port::{BchWalletPort, NewWalletInfo, WalletPortError};

/// BCH 지갑 어댑터
/// bip39 + bitcoin + cashaddr 라이브러리로 실제 HD 지갑을 생성하고
/// AES-GCM으로 개인키를 암호화하여 반환합니다.
pub struct BchWalletAdapter {
    /// AES-GCM 암호화 키 — KMS/환경변수에서 주입
    encryption_key: [u8; 32],
    /// Electrum API 기본 URL
    api_base_url: String,
}

impl BchWalletAdapter {
    pub fn new(encryption_key: [u8; 32], api_base_url: String) -> Self {
        BchWalletAdapter { encryption_key, api_base_url }
    }
}

#[async_trait]
impl BchWalletPort for BchWalletAdapter {
    async fn create_wallet(&self, user_id: i64) -> Result<NewWalletInfo, WalletPortError> {
        todo!("bip39::Mnemonic::generate_in(Language::English, 12) → HD key 파생 → cashaddr 주소 생성 → AES-GCM 암호화 → NewWalletInfo 반환")
    }

    async fn validate_address(&self, address: &str) -> Result<bool, WalletPortError> {
        todo!("cashaddr::decode(address) 성공 여부 확인")
    }

    async fn estimate_fee(&self, amount_satoshi: u64) -> Result<u64, WalletPortError> {
        todo!("Electrum API GET /fee-rate → 수수료 계산")
    }

    async fn broadcast_transaction(
        &self,
        from_address: &str,
        to_address: &str,
        amount_satoshi: u64,
        fee_satoshi: u64,
        encrypted_privkey: &[u8],
    ) -> Result<String, WalletPortError> {
        todo!("AES-GCM 복호화 → bitcoin::PrivateKey 복원 → 오프라인 트랜잭션 서명 → Electrum API broadcast → tx_hash 반환")
    }
}
