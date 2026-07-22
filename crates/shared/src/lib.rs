// crates/shared/src/lib.rs
// 목적: shared 크레이트 루트. frontend와 backend 모두 이 크레이트를 의존하며
//       DTO와 공통 타입을 통해 API 계약을 강제합니다.
pub mod dto;
pub mod types;

pub use types::OpaqueId;
