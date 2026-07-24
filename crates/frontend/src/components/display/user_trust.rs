// crates/frontend/src/components/display/user_trust.rs
// 목적: 사용자 신뢰도 표시 — 퍼센트 프로그레스 바 + 수치

use leptos::prelude::*;

#[component]
pub fn UserTrustIndicator(reliability_index: f64) -> impl IntoView {
    // reliability_index: 0.0 ~ 1.0 → 퍼센트로 변환
    let pct = (reliability_index * 100.0).round().min(100.0) as u32;
    let bar_width = format!("{}%", pct);

    let bar_class = match pct {
        90..=100 => "trust-bar trust-bar-excellent",
        70..=89  => "trust-bar trust-bar-good",
        50..=69  => "trust-bar trust-bar-fair",
        _        => "trust-bar trust-bar-poor",
    };

    view! {
        <div class="trust-indicator">
            <div class="trust-bar-track" aria-label=format!("신뢰도 {}%", pct)>
                <div class=bar_class style=format!("width: {}", bar_width)></div>
            </div>
            <span class="trust-score">{format!("신뢰도 {}%", pct)}</span>
        </div>
    }
}
