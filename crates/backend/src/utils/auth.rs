// crates/backend/src/utils/auth.rs
// 목적: JWT 토큰 발급/검증 유틸리티 및 Claims 타입 정의.
//
// [보안 설계]
// - Claims는 DB 내부 ID(i64)를 포함하지 않고 user_uid(OpaqueId)를 담습니다.
// - user_role 필드로 미들웨어에서 RBAC를 수행합니다.
// - exp(만료 시간) 필드로 토큰 수명을 강제합니다.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("토큰 생성 실패: {0}")]
    TokenCreation(String),
    #[error("토큰 검증 실패: {0}")]
    TokenValidation(String),
    #[error("토큰 만료됨")]
    TokenExpired,
    #[error("권한 없음: 필요 역할 = {required}, 현재 역할 = {actual}")]
    InsufficientRole { required: String, actual: String },
}

/// 사용자 역할 Enum — DB의 VARCHAR role 컬럼을 타입 안전하게 표현
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    User,
    Admin,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::User => write!(f, "user"),
            UserRole::Admin => write!(f, "admin"),
        }
    }
}

/// JWT Payload (Claims)
/// 주의: 내부 DB ID(i64)는 절대 포함되지 않습니다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,       // user_uid (OpaqueId) — DB id 아님
    pub role: UserRole,    // RBAC용 역할
    pub status: String,    // 계정 상태 — 미들웨어에서 dormant/suspended 차단
    pub exp: u64,          // Unix timestamp 만료 시간
    pub iat: u64,          // Unix timestamp 발급 시간
}

/// JWT 서명 비밀 키 — AppState에 SecretString으로 보관
/// 이 구조체는 로그에 절대 출력되지 않도록 Debug를 커스텀 구현
pub struct JwtSecret(pub super::security::SecretString);

impl std::fmt::Debug for JwtSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JwtSecret([REDACTED])")
    }
}

/// JWT 토큰 발급.
/// AppState에서 JwtSecret을 주입받아 호출합니다.
pub fn issue_token(claims: Claims, secret: &JwtSecret) -> Result<String, AuthError> {
    todo!("jsonwebtoken::encode(&Header::default(), &claims, &EncodingKey::from_secret) 호출")
}

/// JWT 토큰 검증 및 Claims 추출.
/// 미들웨어 auth::authenticate에서 호출합니다.
pub fn verify_token(token: &str, secret: &JwtSecret) -> Result<Claims, AuthError> {
    todo!("jsonwebtoken::decode::<Claims>(token, &DecodingKey::from_secret, &Validation::default()) 호출")
}
