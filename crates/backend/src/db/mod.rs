// crates/backend/src/db/mod.rs
// 목적: SeaORM Entity 모듈 진입점.
// 기존 개별 파일들은 entity/ 하위로 이동하여 관리합니다.
// db 계층은 순수 DB 스키마 매핑만 담당하며 비즈니스 로직은 service/ 계층에 위치합니다.
pub mod entity;
