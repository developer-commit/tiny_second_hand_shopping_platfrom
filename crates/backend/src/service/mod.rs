// crates/backend/src/service/mod.rs
// 목적: 비즈니스 로직 서비스 모듈 집합.
// 각 서비스는 Port trait을 통해 외부 인프라에 의존하며,
// Handler로부터 DTO를 받아 처리 후 DTO를 반환합니다.
pub mod auth_service;
pub mod user_service;
pub mod product_service;
pub mod wallet_service;
pub mod escrow_service;
pub mod chat_service;
pub mod review_service;
pub mod report_service;
pub mod admin_service;
pub mod notification_service;
