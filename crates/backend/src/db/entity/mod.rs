// crates/backend/src/db/entity/mod.rs
// 목적: 모든 SeaORM Entity 모듈을 re-export.
// 각 Entity는 DB 스키마와 1:1 대응하며 TryFrom<Model> 변환 트레잇으로 DTO와 분리됩니다.
pub mod chat_message;
pub mod chat_participant;
pub mod chat_room;
pub mod escrow_trade;
pub mod notification;
pub mod platform_stats;
pub mod product;
pub mod product_image;
pub mod product_tag;
pub mod report;
pub mod review;
pub mod user;
pub mod wallet;
pub mod wallet_transaction;
