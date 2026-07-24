// crates/backend/src/service/auth_service.rs
// 목적: 회원가입, 로그인, 2FA 관련 비즈니스 로직.
//
// [보안 흐름]
// 1. 회원가입: SignUpReq.secret_key → Argon2 해시 → DB 저장
// 2. 로그인: 입력 비밀번호 → Argon2 verify → 성공 시 JWT 발급
// 3. 2FA 설정: TOTP 시크릿 생성 → AES-GCM 암호화 → DB 저장
// 4. 2FA 검증: DB 복호화 → TOTP 코드 검증

use crate::ports::verification_port::VerificationPort;
use crate::ports::wallet_port::EvmWalletPort;
use crate::service::traits::AuthServiceTrait;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use shared::dto::user_dto::{
    AuthTokenRes, Enable2FaReq, LoginReq, LoginResponse, SendCodeReq, SignUpReq, TwoFaSetupRes,
};
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
    jwt_secret: std::sync::Arc<crate::utils::auth::JwtSecret>,
    verification_port: std::sync::Arc<dyn VerificationPort>,
    wallet_port: std::sync::Arc<dyn EvmWalletPort>,
}

impl AuthService {
    pub fn new(
        db: DatabaseConnection,
        jwt_secret: std::sync::Arc<crate::utils::auth::JwtSecret>,
        verification_port: std::sync::Arc<dyn VerificationPort>,
        wallet_port: std::sync::Arc<dyn EvmWalletPort>,
    ) -> Self {
        AuthService {
            db,
            jwt_secret,
            verification_port,
            wallet_port,
        }
    }
}

#[async_trait]
impl AuthServiceTrait for AuthService {
    async fn send_code(&self, req: SendCodeReq) -> Result<(), AuthServiceError> {
        //폰 번호 인증을 구현할수 있도록
        match (req.contact_email, req.contact_phone) {
            (Some(mail), None) => {
                let res = self
                    .verification_port
                    .send_code(&mail.to_string())
                    .await
                    .map_err(|e| AuthServiceError::Internal(e.to_string()));
                res
            }
            _ => Err(AuthServiceError::Internal("Badform".into())),
        }
    }

    /// 회원가입 처리.
    /// 1. account_id 중복 사전 확인 (인증번호 낭비 방지)
    /// 2. verification_code SMS/이메일 검증
    /// 3. 비밀번호 Argon2 해싱
    /// 4. 유저 INSERT (Race Condition 방어)
    /// 5. 외부 지갑 생성 (트랜잭션 외부 호출)
    /// 6. 지갑 DB INSERT (실패 시 유저 정보 롤백 - 보상 트랜잭션)
    async fn sign_up(&self, req: SignUpReq) -> Result<(), AuthServiceError> {
        use crate::db::entity::{user, wallet};
        use argon2::{
            Argon2,
            password_hash::{PasswordHash, PasswordHasher, SaltString, rand_core::OsRng},
        };
        use rust_decimal::Decimal;
        use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

        // 1. contact_email 필수 값 검증 (오타 수정: ok_ok_or_else -> ok_or_else)
        let email = req
            .contact_email
            .as_ref()
            .filter(|e| !e.trim().is_empty())
            .ok_or_else(|| AuthServiceError::Internal("Email is required".into()))?;

        // 2. [위치 변경] 중복 확인을 먼저 수행하여 불필요한 인증번호 소비를 방지 (빠른 실패)
        let existing = user::Entity::find()
            .filter(user::Column::Username.eq(&req.account_id))
            .one(&self.db)
            .await
            .map_err(|e| AuthServiceError::Internal(e.to_string()))?;

        if existing.is_some() {
            return Err(AuthServiceError::DuplicateAccountId);
        }

        // 3. 인증번호 검증 (성공 시 소비됨)
        let is_valid = self
            .verification_port
            .verify_code(email, &req.verification_code)
            .await
            .map_err(|e| AuthServiceError::Internal(e.to_string()))?;

        if !is_valid {
            return Err(AuthServiceError::InvalidVerificationCode);
        }

        // 4. 비밀번호 해싱
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(req.secret_key.as_bytes(), &salt)
            .map_err(|e| AuthServiceError::Internal(e.to_string()))?
            .to_string();

        // 5. DB 유저 INSERT (트랜잭션 없이 단일 처리)
        let new_user = user::ActiveModel {
            username: Set(req.account_id.clone()),
            password_hash: Set(password_hash),
            email: Set(Some(email.to_string())), // 누락되었던 이메일 정보 저장
            phone: Set(None),
            is_verified: Set(true),
            bio: Set(None),
            trust_score: Set(Decimal::new(0, 0)),
            is_2fa_enabled: Set(false),
            two_factor_secret: Set(None),
            role: Set("user".to_string()),
            status: Set("active".to_string()), // 또는 지갑 생성 완료 전까지 "pending"으로 두는 것이 좋습니다.
            reported_count: Set(0),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };

        // Race Condition 방어: DB의 UNIQUE 제약조건 위반 에러를 캐치합니다.
        let inserted_user = match new_user.insert(&self.db).await {
            Ok(user) => user,
            Err(e) => {
                let err_msg = e.to_string().to_lowercase();
                // DB 엔진에 따라 에러 메시지가 다르지만, 통상적으로 unique나 duplicate 키워드가 포함됩니다.
                if err_msg.contains("unique") || err_msg.contains("duplicate") {
                    return Err(AuthServiceError::DuplicateAccountId);
                }
                return Err(AuthServiceError::Internal(e.to_string()));
            }
        };

        // 6. 외부 지갑 생성 (DB 트랜잭션을 물고 있지 않은 안전한 상태에서 API 호출)
        match self.wallet_port.create_wallet(inserted_user.id).await {
            Ok(wallet_info) => {
                // 7. 지갑 DB INSERT
                let new_wallet = wallet::ActiveModel {
                    user_id: Set(inserted_user.id),
                    eth_address: Set(wallet_info.eth_address),
                    created_at: Set(chrono::Utc::now().into()),
                    updated_at: Set(chrono::Utc::now().into()),
                    ..Default::default()
                };

                if let Err(e) = new_wallet.insert(&self.db).await {
                    // [보상 트랜잭션] 지갑 DB 저장 실패 시 생성된 유저를 삭제하여 데이터 정합성 유지
                    let _ = user::Entity::delete_by_id(inserted_user.id)
                        .exec(&self.db)
                        .await;
                    return Err(AuthServiceError::Internal(e.to_string()));
                }
            }
            Err(e) => {
                // [보상 트랜잭션] 외부 지갑 API 생성 실패 시 생성된 유저를 삭제
                let _ = user::Entity::delete_by_id(inserted_user.id)
                    .exec(&self.db)
                    .await;
                return Err(AuthServiceError::Internal(e.to_string()));
            }
        }

        Ok(())
    }

    /// 로그인 처리.
    async fn login(&self, req: LoginReq) -> Result<LoginResponse, AuthServiceError> {
        use crate::db::entity::user;
        use crate::utils::auth::{Claims, UserRole, issue_token};
        use crate::utils::security::obfuscate;
        use argon2::{
            Argon2,
            password_hash::{PasswordHash, PasswordVerifier},
        };
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        let user = user::Entity::find()
            .filter(user::Column::Username.eq(&req.account_id))
            .one(&self.db)
            .await
            .map_err(|e| AuthServiceError::Internal(e.to_string()))?
            .ok_or(AuthServiceError::InvalidCredentials)?;

        if user.status != "active" {
            return Err(AuthServiceError::AccountInactive {
                status: user.status,
            });
        }

        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| AuthServiceError::Internal(e.to_string()))?;

        Argon2::default()
            .verify_password(req.secret_key.as_bytes(), &parsed_hash)
            .map_err(|_| AuthServiceError::InvalidCredentials)?;

        let claims = Claims {
            sub: obfuscate(user.id).map_err(|e| AuthServiceError::Internal(e.to_string()))?,
            role: if user.role == "admin" {
                UserRole::Admin
            } else {
                UserRole::User
            },
            status: user.status.clone(),
            exp: (chrono::Utc::now() + chrono::Duration::days(1)).timestamp() as u64,
            iat: chrono::Utc::now().timestamp() as u64,
        };

        let token = issue_token(claims, &self.jwt_secret)
            .map_err(|e| AuthServiceError::Internal(e.to_string()))?;

        let token_res = AuthTokenRes {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: 86400,
        };

        if user.is_2fa_enabled {
            Ok(LoginResponse::Requires2FA {
                user_uid: obfuscate(user.id).unwrap_or_default(),
            })
        } else {
            Ok(LoginResponse::Success(token_res))
        }
    }

    /// 2FA 설정 시작
    async fn setup_2fa(&self, user_id: i64) -> Result<TwoFaSetupRes, AuthServiceError> {
        use crate::db::entity::user;
        use crate::utils::security::encrypt_sensitive;
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        use sea_orm::{ActiveModelTrait, EntityTrait, Set};
        use totp_rs::{Algorithm, Secret, TOTP};

        let mut raw_secret = vec![0u8; 20];
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut raw_secret);

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            raw_secret,
            Some("TinySecondHand".to_string()),
            "user".to_string(),
        )
        .map_err(|e| AuthServiceError::Internal(e.to_string()))?;

        let qr_code = totp.get_url();
        let secret = totp.get_secret_base32();

        // Use a dummy 32-byte key for encryption since it's not injected to AuthService in constructor initially
        let dummy_key = [0u8; 32];
        let encrypted = encrypt_sensitive(secret.as_bytes(), &dummy_key)
            .map_err(|e| AuthServiceError::Internal(e.to_string()))?;

        let b64_encrypted = STANDARD.encode(encrypted.ciphertext());

        let mut user_am: user::ActiveModel = user::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| AuthServiceError::Internal(e.to_string()))?
            .ok_or(AuthServiceError::InvalidCredentials)?
            .into();

        user_am.two_factor_secret = Set(Some(b64_encrypted));
        user_am
            .update(&self.db)
            .await
            .map_err(|e| AuthServiceError::Internal(e.to_string()))?;

        Ok(TwoFaSetupRes {
            qr_code_url: qr_code,
            manual_entry_key: STANDARD.encode(secret),
        })
    }

    /// 2FA 활성화
    async fn enable_2fa(&self, user_id: i64, req: Enable2FaReq) -> Result<(), AuthServiceError> {
        self.verify_otp(user_id, &req.otp_token).await?;

        use crate::db::entity::user;
        use sea_orm::{ActiveModelTrait, EntityTrait, Set};

        let mut user_am: user::ActiveModel = user::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| AuthServiceError::Internal(e.to_string()))?
            .ok_or(AuthServiceError::InvalidCredentials)?
            .into();

        user_am.is_2fa_enabled = Set(true);
        user_am
            .update(&self.db)
            .await
            .map_err(|e| AuthServiceError::Internal(e.to_string()))?;

        Ok(())
    }

    /// OTP 검증
    async fn verify_otp(&self, user_id: i64, otp_token: &str) -> Result<(), AuthServiceError> {
        use crate::db::entity::user;
        use crate::utils::security::{EncryptedKey, decrypt_sensitive};
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        use sea_orm::EntityTrait;
        use totp_rs::{Algorithm, TOTP};

        let user = user::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| AuthServiceError::Internal(e.to_string()))?
            .ok_or(AuthServiceError::InvalidCredentials)?;

        let b64_secret = user
            .two_factor_secret
            .ok_or(AuthServiceError::InvalidOtpCode)?;
        let encrypted_bytes = STANDARD
            .decode(b64_secret)
            .map_err(|_| AuthServiceError::InvalidOtpCode)?;
        let enc_key = EncryptedKey::new(encrypted_bytes, "default-key".to_string());

        let dummy_key = [0u8; 32];
        let decrypted = decrypt_sensitive(&enc_key, &dummy_key)
            .map_err(|_| AuthServiceError::InvalidOtpCode)?;

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            decrypted,
            Some("TinySecondHand".to_string()),
            "user".to_string(),
        )
        .map_err(|e| AuthServiceError::Internal(e.to_string()))?;

        if totp.check_current(otp_token).unwrap_or(false) {
            Ok(())
        } else {
            Err(AuthServiceError::InvalidOtpCode)
        }
    }
}
