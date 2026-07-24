// crates/shared/src/dto/mod.rs
// 목적: 모든 API DTO 모듈의 진입점.
// 각 모듈은 openapi.yaml의 스키마 은닉 원칙을 따라
// DB 컬럼명과 다른 필드명을 사용합니다.

pub mod admin_dto;
pub mod chat_dto;
pub mod common_dto;
pub mod error_dto;
pub mod escrow_dto;
pub mod noti_dto;
pub mod product_dto;
pub mod report_dto;
pub mod review_dto;
pub mod transaction_dto;
pub mod user_dto;
