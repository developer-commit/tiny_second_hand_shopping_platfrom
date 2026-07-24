// crates/frontend/src/components/input/price_input.rs
// 목적: ETH 금액 입력 — 소수점 처리 + "ETH" 접미사 표시

use leptos::prelude::*;

#[component]
pub fn PriceInput(
    signal: RwSignal<f64>,
    #[prop(optional)]
    error: Option<Signal<Option<String>>>,
) -> impl IntoView {
    // 내부적으로 String 버퍼를 유지하여 사용자 입력 경험을 자연스럽게 합니다.
    let display_val = RwSignal::new(format!("{}", signal.get_untracked()));

    let has_error = move || {
        error.map(|e| e.get().is_some()).unwrap_or(false)
    };
    let error_msg = move || error.and_then(|e| e.get());

    let on_input = move |ev: leptos::ev::Event| {
        let raw = event_target_value(&ev);
        display_val.set(raw.clone());
        if let Ok(parsed) = raw.parse::<f64>() {
            signal.set(parsed);
        }
    };

    view! {
        <div class="form-field">
            <label class="form-label" for="price-input">"가격 (ETH)"</label>
            <div class="price-input-wrapper">
                <input
                    id="price-input"
                    type="number"
                    step="0.0001"
                    min="0"
                    class=move || if has_error() { "form-input price-input form-input-error" } else { "form-input price-input" }
                    placeholder="0.0000"
                    prop:value=move || display_val.get()
                    on:input=on_input
                />
                <span class="price-input-suffix">"ETH"</span>
            </div>
            <Show when=has_error>
                <span class="form-error-msg" role="alert">
                    {move || error_msg().unwrap_or_default()}
                </span>
            </Show>
        </div>
    }
}
