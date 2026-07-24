// crates/shared/src/dto/user_dto.rs
// 목적: 회원가입, 로그인, 프로필 관련 Request/Response DTO.
// 보안 원칙:
// - DB 컬럼명(username, password_hash, trust_score, is_2fa_enabled)을
//   openapi.yaml 기준의 은닉 명칭(display_name, secret_key, reliability_index,
//   require_otp)으로 완전히 교체합니다.
// - secret_key(비밀번호)는 이 DTO에서만 String으로 수신하며,
//   Service 계층에서 즉시 SecretString으로 변환 후 소멸시킵니다.

use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::types::OpaqueId;

// ─── Auth Request DTOs ──────────────────────────────────────────────────────

/// [Request] POST /auth/sendcode — 인증메일 전송
/// openapi: account_id(username),  contact_email(email), contact_phone(phone)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SendCodeReq {
    #[validate(length(min = 3, max = 50))]
    pub account_id: String,       // DB: username
    #[validate(email)]
    pub contact_email: Option<String>,  // DB: email
    #[validate(length(min = 10, max = 20))]
    pub contact_phone: Option<String>,  // DB: phone
}

/// [Request] POST /auth/signup — 회원가입
/// openapi: account_id(username), secret_key(password), contact_email(email), contact_phone(phone)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SignUpReq {
    #[validate(length(min = 3, max = 50))]
    pub account_id: String,       // DB: username
    #[validate(length(min = 8, max = 128))]
    pub secret_key: String,       // DB: password_hash (plaintext, 즉시 해싱)
    #[validate(email)]
    pub contact_email: Option<String>,  // DB: email
    #[validate(length(min = 10, max = 20))]
    pub contact_phone: Option<String>,  // DB: phone
    pub verification_code: String, // SMS/이메일 인증코드
}

/// [Request] POST /auth/login — 로그인
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginReq {
    #[validate(length(min = 3, max = 50))]
    pub account_id: String,  // DB: username
    pub secret_key: String,  // plaintext password
}

/// [Response] POST /auth/login — JWT 토큰 발급
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokenRes {
    pub access_token: String,  // JWT Bearer Token
    pub token_type: String,    // "Bearer"
    pub expires_in: u64,       // 초 단위 만료 시간
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
pub enum LoginResponse {
    Success(AuthTokenRes),
    Requires2FA { user_uid: crate::types::OpaqueId },
}

// ─── User Profile DTOs ──────────────────────────────────────────────────────

/// [Response] GET /users/me — 내 프로필
/// 스키마 은닉: DB 컬럼명 → API 필드명
/// username → display_name, trust_score → reliability_index, is_2fa_enabled → require_otp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileRes {
    pub user_uid: OpaqueId,          // DB: id (난독화)
    pub display_name: String,        // DB: username
    pub contact_email: Option<String>, // DB: email
    pub bio: Option<String>,         // DB: bio
    pub reliability_index: f64,      // DB: trust_score
    pub require_otp: bool,           // DB: is_2fa_enabled
    pub account_status: String,      // DB: status (active/dormant/suspended)
    pub joined_at: String,           // DB: created_at
}

/// [Request] PATCH /users/me — 프로필 수정
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateProfileReq {
    #[validate(length(max = 200))]
    pub bio: Option<String>,         // DB: bio
}

/// [Request] POST /users/me/2fa/enable — 2FA 활성화
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enable2FaReq {
    pub otp_token: String,           // 검증용 OTP 코드
}

/// [Response] POST /users/me/2fa/setup — 2FA 설정 시작 (QR 코드 URL 반환)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFaSetupRes {
    pub qr_code_url: String,         // TOTP QR 코드
    pub manual_entry_key: String,    // 수동 입력용 시크릿 (마스킹 처리)
}
