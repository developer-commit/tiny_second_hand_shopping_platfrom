// crates/frontend/src/components/input/form_input.rs
// 목적: 공용 입력 필드 — 레이블 + input + 인라인 에러 메시지
// signal 바인딩으로 양방향 데이터 흐름을 구현합니다.

use leptos::prelude::*;

#[component]
pub fn FormInput(
    /// 입력 필드 레이블
    #[prop(into)]
    label: String,
    /// placeholder 텍스트
    #[prop(into, default = String::new())]
    placeholder: String,
    /// input type (text / password / email / number)
    #[prop(into, default = "text".to_string())]
    input_type: String,
    /// 양방향 바인딩 Signal
    signal: RwSignal<String>,
    /// 인라인 에러 메시지 (None이면 숨김)
    #[prop(optional)]
    error: Option<Signal<Option<String>>>,
    /// HTML id (label for 연결용)
    #[prop(into, default = String::new())]
    id: String,
) -> impl IntoView {
    let input_id = if id.is_empty() {
        // label을 snake_case로 변환하여 id 자동 생성
        label.to_lowercase().replace(' ', "_")
    } else {
        id
    };

    let has_error = move || error.map(|e| e.get().is_some()).unwrap_or(false);
    let error_msg = move || error.and_then(|e| e.get());

    view! {
        <div class="form-field">
            <label class="form-label" for=input_id.clone()>
                {label}
            </label>
            <input
                id=input_id
                type=input_type
                class=move || if has_error() { "form-input form-input-error" } else { "form-input" }
                placeholder=placeholder
                prop:value=move || signal.get()
                on:input=move |ev| signal.set(event_target_value(&ev))
                aria-invalid=move || has_error().to_string()
            />
            <Show when=has_error>
                <span class="form-error-msg" role="alert">
                    {move || error_msg().unwrap_or_default()}
                </span>
            </Show>
        </div>
    }
}
