// crates/backend/src/ports/mod.rs
// 목적: Hexagonal Architecture의 Port(인터페이스) 모듈 집합.
// Service 계층은 오직 이 Port trait에만 의존하며,
// 실제 구현(DB, BCH 네트워크, Redis)은 infra 계층에서만 처리됩니다.
pub mod wallet_port;
pub mod bch_network_port;
pub mod pubsub_port;
pub mod notification_port;
pub mod verification_port;
