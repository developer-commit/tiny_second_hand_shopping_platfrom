// crates/frontend/src/components/input/otp_input.rs
// 목적: N자리 OTP 입력 — 각 자리 개별 input, 입력 후 자동 다음 칸 포커스
// wasm_bindgen을 통해 DOM에 직접 접근하여 focus를 이동합니다.

use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn OtpInput(
    signal: RwSignal<String>,
    #[prop(default = 6_usize)]
    length: usize,
) -> impl IntoView {
    // 각 자리를 개별 RwSignal로 관리
    let digits: Vec<RwSignal<String>> = (0..length)
        .map(|_| RwSignal::new(String::new()))
        .collect();

    // digits 변화 시 signal 동기화
    {
        let digits_clone = digits.clone();
        Effect::new(move |_| {
            let combined: String = digits_clone.iter().map(|d| d.get()).collect();
            signal.set(combined);
        });
    }

    view! {
        <div class="otp-input-wrapper" role="group" aria-label="OTP 코드 입력">
            {digits.into_iter().enumerate().map(|(i, digit)| {
                let input_id = format!("otp-{}", i);
                let next_id  = format!("otp-{}", i + 1);

                let on_input = move |ev: leptos::ev::Event| {
                    let raw = event_target_value(&ev);
                    // 숫자만 허용, 1자리만
                    let ch: String = raw.chars().filter(|c| c.is_ascii_digit()).take(1).collect();
                    digit.set(ch.clone());

                    // 입력 완료 시 다음 칸으로 포커스 이동
                    if !ch.is_empty() {
                        if let Some(window) = web_sys::window() {
                            if let Some(document) = window.document() {
                                if let Some(next_el) = document.get_element_by_id(&next_id) {
                                    if let Ok(input) = next_el.dyn_into::<web_sys::HtmlInputElement>() {
                                        let _ = input.focus();
                                    }
                                }
                            }
                        }
                    }
                };

                let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
                    // Backspace 시 현재 칸 비우고 이전 칸으로 이동
                    if ev.key() == "Backspace" && digit.get().is_empty() && i > 0 {
                        let prev_id = format!("otp-{}", i - 1);
                        if let Some(window) = web_sys::window() {
                            if let Some(document) = window.document() {
                                if let Some(prev_el) = document.get_element_by_id(&prev_id) {
                                    if let Ok(input) = prev_el.dyn_into::<web_sys::HtmlInputElement>() {
                                        let _ = input.focus();
                                    }
                                }
                            }
                        }
                    }
                };

                view! {
                    <input
                        id=input_id
                        type="text"
                        inputmode="numeric"
                        pattern="[0-9]"
                        maxlength="1"
                        class="otp-digit"
                        prop:value=move || digit.get()
                        on:input=on_input
                        on:keydown=on_keydown
                        autocomplete="one-time-code"
                        aria-label=format!("OTP 코드 {}번째 자리", i + 1)
                    />
                }
            }).collect_view()}
        </div>
    }
}
