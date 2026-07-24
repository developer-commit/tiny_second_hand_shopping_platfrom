// crates/backend/src/db/entity/user.rs
// 목적: users 테이블의 SeaORM Entity.
//
// [보안 설계]
// - two_factor_secret은 평문 String이 아닌 DB에는 암호화된 바이트로 저장.
//   Entity 필드 타입은 String(Base64 인코딩된 암호문)이며,
//   서비스 계층에서 EncryptedKey로 래핑하여 처리합니다.
// - password_hash는 Argon2 해시값이므로 Entity에서 String으로 보관 가능.
//   단, DTO로 변환 시 절대 포함되지 않습니다.
// - is_verified, is_2fa_enabled, role, status 등 보안 상태 필드는
//   Model에만 존재하고 외부 DTO에는 은닉/변환됩니다.

use crate::utils::security::{SecurityError, obfuscate};
use sea_orm::entity::prelude::*;
use shared::dto::user_dto::UserProfileRes;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64, // 내부 PK — 외부 노출 금지
    pub username: String,      // 아이디 (유일)
    pub password_hash: String, // Argon2 해시 — DTO 제외
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_verified: bool,
    pub bio: Option<String>,
    pub trust_score: Decimal, // 신뢰도 지수
    pub is_2fa_enabled: bool,
    pub two_factor_secret: Option<String>, // AES-GCM 암호문(Base64) — DTO 제외
    pub role: String,                      // "user" | "admin"
    pub status: String,                    // "active" | "dormant" | "suspended"
    pub reported_count: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::wallet::Entity")]
    Wallet,
    #[sea_orm(has_many = "super::product::Entity")]
    Products,
    #[sea_orm(has_many = "super::escrow_trade::Entity")]
    EscrowTrades,
}

impl ActiveModelBehavior for ActiveModel {}

// ─── DTO 변환 ────────────────────────────────────────────────────────────────

impl TryFrom<Model> for UserProfileRes {
    type Error = SecurityError;

    /// DB Model → 외부 DTO 변환.
    /// password_hash, two_factor_secret은 변환 결과에 절대 포함되지 않습니다.
    fn try_from(m: Model) -> Result<Self, Self::Error> {
        Ok(UserProfileRes {
            user_uid: obfuscate(m.id)?, // i64 PK → OpaqueId
            display_name: m.username,   // username → display_name
            contact_email: m.email,
            bio: m.bio,
            reliability_index: m.trust_score.try_into().unwrap_or(0.0), // trust_score → reliability_index
            require_otp: m.is_2fa_enabled, // is_2fa_enabled → require_otp
            account_status: m.status,
            joined_at: m.created_at.to_rfc3339(),
        })
    }
}
