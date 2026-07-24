// crates/frontend/src/components/feedback/qr_code.rs
// 목적: QR 코드 이미지 표시 — 외부 QR API 사용 (크레이트 추가 없이 구현)
// data: ETH 주소 또는 TOTP URI를 인코딩합니다.

use leptos::prelude::*;

const QR_API_BASE: &str = "https://api.qrserver.com/v1/create-qr-code/";

/// 간단한 URL 퍼센트 인코딩 (RFC 3986 unreserved 문자만 패스스루)
fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 3);
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            b => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

#[component]
pub fn QrCodeDisplay(
    #[prop(into)] data: String,
    /// QR 이미지 크기 (px)
    #[prop(default = 200_u32)]
    size: u32,
) -> impl IntoView {
    let encoded = percent_encode(&data);
    let preview_len = data.len().min(30);
    let alt = format!("QR 코드: {}", &data[..preview_len]);
    let src = format!(
        "{}?size={}x{}&data={}&margin=2&format=png",
        QR_API_BASE, size, size, encoded
    );

    view! {
        <div class="qr-code-wrapper">
            <img
                src=src
                alt=alt
                width=size
                height=size
                class="qr-code-img"
                loading="lazy"
            />
        </div>
    }
}
