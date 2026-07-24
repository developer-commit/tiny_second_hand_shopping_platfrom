// crates/backend/src/utils/security.rs
// 목적: 3대 보안 원칙 중 1번(식별자 난독화)과 3번(인증 보안 타입화)을 구현합니다.
//
// [보안 설계]
// 1. ID 난독화: 내부 i64 PK ↔ 외부 공개용 String 변환 (sqids 방식 권장)
// 2. SecretString: 비밀번호·시크릿을 감싸는 래퍼. Debug 미구현으로 로그 유출 방지
// 3. EncryptedKey: AES-256-GCM 암호화된 지갑 개인키 래퍼.
//    Clone 미구현으로 복사 전파 최소화

use thiserror::Error;

// ─── 커스텀 에러 ────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("ID 난독화 실패: {0}")]
    ObfuscationFailed(String),
    #[error("ID 역난독화 실패 — 유효하지 않은 식별자: {0}")]
    DeobfuscationFailed(String),
    #[error("암호화 실패: {0}")]
    EncryptionFailed(String),
    #[error("복호화 실패 — 키 또는 데이터 손상")]
    DecryptionFailed,
    #[error("키 길이 불일치")]
    InvalidKeyLength,
}

// ─── ID 난독화 유틸리티 ──────────────────────────────────────────────────────

/// 내부 순차 ID(i64)를 외부에 안전한 불투명 문자열로 변환.
/// 내부 구현은 Sqids(https://sqids.org/) 또는 동등한 알고리즘을 사용합니다.
///
/// # 사용처
/// DB에서 조회한 i64 PK → DTO의 OpaqueId 필드로 변환 시
pub fn obfuscate(internal_id: i64) -> Result<String, SecurityError> {
    sqids::Sqids::default()
        .encode(&[internal_id as u64])
        .map_err(|e| SecurityError::ObfuscationFailed(e.to_string()))
}

/// 외부 불투명 문자열을 내부 순차 ID(i64)로 역변환.
/// 역변환 실패 시 SecurityError::DeobfuscationFailed 반환 (존재하지 않는 리소스처럼 처리).
///
/// # 사용처
/// 핸들러에서 Path 파라미터(OpaqueId)를 DB 조회용 i64로 변환 시
pub fn deobfuscate(public_uid: &str) -> Result<i64, SecurityError> {
    let ids = sqids::Sqids::default().decode(public_uid);
    if ids.is_empty() {
        return Err(SecurityError::DeobfuscationFailed(public_uid.to_string()));
    }
    Ok(ids[0] as i64)
}

// ─── 보안 래퍼 타입 ──────────────────────────────────────────────────────────

/// 비밀번호, JWT 시크릿, 2FA 시크릿 키 등 평문 민감 문자열 래퍼.
///
/// # 보안 특성
/// - `Debug` 미구현 → tracing/serde 로그에 절대 노출되지 않음
/// - `Clone` 미구현 → 불필요한 복사 전파 방지
/// - `Drop` 시 메모리를 0으로 채우는 zeroize 패턴 적용 권장
pub struct SecretString(String);

impl SecretString {
    pub fn new(s: String) -> Self {
        SecretString(s)
    }
    /// 내부 값 접근 — 반드시 필요한 처리 직후 소멸하도록 설계
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

// Debug, Clone, Serialize를 의도적으로 구현하지 않음
impl std::fmt::Display for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

/// AES-256-GCM으로 암호화된 지갑 개인키 또는 2FA 시크릿 저장 타입.
/// DB에는 이 타입의 `ciphertext` 바이트만 저장합니다.
///
/// # 보안 특성
/// - `Debug` 미구현 → 로그 유출 방지
/// - `Clone` 미구현 → 복사 전파 방지
/// - 복호화는 KMS 키를 보유한 infra 계층에서만 수행
pub struct EncryptedKey {
    ciphertext: Vec<u8>,    // AES-GCM 암호문 + nonce
    key_id: String,         // 어떤 KMS 키로 암호화했는지 참조
}

impl EncryptedKey {
    pub fn new(ciphertext: Vec<u8>, key_id: String) -> Self {
        EncryptedKey { ciphertext, key_id }
    }
    pub fn ciphertext(&self) -> &[u8] {
        &self.ciphertext
    }
    pub fn key_id(&self) -> &str {
        &self.key_id
    }
}

impl std::fmt::Debug for EncryptedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EncryptedKey([REDACTED], key_id: {})", self.key_id)
    }
}

// ─── 암호화 유틸리티 ─────────────────────────────────────────────────────────

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

/// AES-256-GCM으로 민감 데이터 암호화.
/// 2FA 시크릿 키 및 지갑 개인키를 DB에 저장하기 전에 호출합니다.
pub fn encrypt_sensitive(plaintext: &[u8], key: &[u8; 32]) -> Result<EncryptedKey, SecurityError> {
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| SecurityError::EncryptionFailed(e.to_string()))?;
        
    let mut final_data = nonce_bytes.to_vec();
    final_data.extend_from_slice(&ciphertext);
    
    Ok(EncryptedKey::new(final_data, "default-key-id".to_string()))
}

/// AES-256-GCM 복호화. infra 계층의 KMS 어댑터에서만 사용.
pub fn decrypt_sensitive(encrypted: &EncryptedKey, key: &[u8; 32]) -> Result<Vec<u8>, SecurityError> {
    let data = encrypted.ciphertext();
    if data.len() < 12 {
        return Err(SecurityError::InvalidKeyLength);
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(key.into());
    
    cipher.decrypt(nonce, ciphertext)
        .map_err(|_| SecurityError::DecryptionFailed)
}
