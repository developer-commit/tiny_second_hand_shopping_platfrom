// crates/shared/src/types.rs
// 목적: 플랫폼 전역 커스텀 타입 정의.
// - OpaqueId: DB의 순차 i64를 외부에 절대 노출하지 않기 위한 타입 별칭.
//   String 기반이므로 serde로 JSON 직렬화가 가능하며,
//   타입 별칭을 사용함으로써 ID 필드임을 코드에서 명확히 표현합니다.

use serde::{Deserialize, Serialize};

/// 외부에 노출되는 난독화된 식별자 타입.
/// 내부 DB의 `i64` primary key를 절대 직접 사용하지 않으며,
/// backend의 `security::obfuscate` / `deobfuscate` 함수를 통해서만 변환합니다.
pub type OpaqueId = String;

/// BCH 금액을 표현하는 타입. f64 대신 Decimal을 사용해야 하지만
/// frontend/shared 계층에서의 표현을 위해 f64 래퍼를 제공합니다.
/// (백엔드 DB/계산 레이어는 rust_decimal을 사용)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct BchAmount(pub f64);

impl BchAmount {
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl From<f64> for BchAmount {
    fn from(v: f64) -> Self {
        BchAmount(v)
    }
}
