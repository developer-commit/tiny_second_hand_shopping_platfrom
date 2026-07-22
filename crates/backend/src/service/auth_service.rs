// crates/backend/src/service/auth_service.rs
// 목적: 회원가입, 로그인, 2FA 관련 비즈니스 로직.
//
// [보안 흐름]
// 1. 회원가입: SignUpReq.secret_key → Argon2 해시 → DB 저장
// 2. 로그인: 입력 비밀번호 → Argon2 verify → 성공 시 JWT 발급
// 3. 2FA 설정: TOTP 시크릿 생성 → AES-GCM 암호화 → DB 저장
// 4. 2FA 검증: DB 복호화 → TOTP 코드 검증

use sea_orm::DatabaseConnection;
use shared::dto::user_dto::{AuthTokenRes, Enable2FaReq, LoginReq, SignUpReq, TwoFaSetupRes};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthServiceError {
    #[error("이미 존재하는 계정 ID입니다.")]
    DuplicateAccountId,
    #[error("인증 코드가 유효하지 않습니다.")]
    InvalidVerificationCode,
    #[error("계정 ID 또는 비밀번호가 올바르지 않습니다.")]
    InvalidCredentials,
    #[error("계정이 비활성화되었습니다: {status}")]
    AccountInactive { status: String },
    #[error("2FA OTP 코드가 유효하지 않습니다.")]
    InvalidOtpCode,
    #[error("내부 서버 오류: {0}")]
    Internal(String),
}

pub struct AuthService {
    db: DatabaseConnection,
    // jwt_secret은 AppState에서 Arc로 공유
}

impl AuthService {
    pub fn new(db: DatabaseConnection) -> Self {
        AuthService { db }
    }

    /// 회원가입 처리.
    /// 1. account_id 중복 확인
    /// 2. verification_code SMS/이메일 검증
    /// 3. 비밀번호 Argon2 해싱
    /// 4. users 테이블 INSERT + wallets 테이블 INSERT (지갑 동시 생성)
    pub async fn sign_up(&self, req: SignUpReq) -> Result<(), AuthServiceError> {
        todo!("중복확인 → 인증코드 검증 → argon2::hash → DB INSERT")
    }

    /// 로그인 처리.
    /// 1. account_id로 사용자 조회
    /// 2. argon2::verify로 비밀번호 검증
    /// 3. 계정 상태(status) 확인 (dormant/suspended 차단)
    /// 4. JWT Claims 생성 및 토큰 발급
    pub async fn login(&self, req: LoginReq) -> Result<AuthTokenRes, AuthServiceError> {
        todo!("DB 조회 → argon2::verify → 상태 확인 → JWT 발급")
    }

    /// 2FA 설정 시작 — TOTP 시크릿 생성 및 QR 코드 URL 반환
    pub async fn setup_2fa(&self, user_id: i64) -> Result<TwoFaSetupRes, AuthServiceError> {
        todo!("totp_rs::TOTP::new → QR 코드 생성 → 시크릿 AES 암호화 → DB 저장")
    }

    /// 2FA 활성화 — OTP 코드 검증 후 is_2fa_enabled = true
    pub async fn enable_2fa(
        &self,
        user_id: i64,
        req: Enable2FaReq,
    ) -> Result<(), AuthServiceError> {
        todo!("DB에서 암호화된 시크릿 복호화 → TOTP 코드 검증 → is_2fa_enabled 업데이트")
    }

    /// 지갑 출금 등 민감 작업 전 OTP 재검증
    pub async fn verify_otp(
        &self,
        user_id: i64,
        otp_token: &str,
    ) -> Result<(), AuthServiceError> {
        todo!("DB 암호화 시크릿 복호화 → totp_rs::TOTP::check_current")
    }
}
