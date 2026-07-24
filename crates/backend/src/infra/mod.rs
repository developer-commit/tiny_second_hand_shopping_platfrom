// crates/backend/src/infra/mod.rs
// 목적: 외부 인프라 어댑터 모듈 집합.
// 각 어댑터는 ports/ 에서 정의한 Port trait을 구현합니다.
pub mod evm_wallet_adapter;
pub mod bch_network_adapter;
pub mod redis_pubsub_adapter;
pub mod verification_adapter;
pub mod mock_verification_adapter;