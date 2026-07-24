// crates/frontend/src/components/input/form_textarea.rs
// 목적: 멀티라인 입력 필드 — 레이블 + textarea + 글자 수 카운터 + 에러

use leptos::prelude::*;

#[component]
pub fn FormTextarea(
    #[prop(into)] label: String,
    #[prop(into, default = String::new())] placeholder: String,
    signal: RwSignal<String>,
    /// 최대 글자 수 (0이면 제한 없음)
    #[prop(default = 0_usize)]
    max_length: usize,
    #[prop(optional)] error: Option<Signal<Option<String>>>,
    #[prop(into, default = String::new())] id: String,
) -> impl IntoView {
    let input_id = if id.is_empty() {
        label.to_lowercase().replace(' ', "_")
    } else {
        id
    };

    let char_count = move || signal.get().chars().count();
    let has_error = move || error.map(|e| e.get().is_some()).unwrap_or(false);
    let error_msg = move || error.and_then(|e| e.get());
    let show_counter = max_length > 0;

    view! {
        <div class="form-field">
            <div class="form-field-header">
                <label class="form-label" for=input_id.clone()>{label}</label>
                <Show when=move || show_counter>
                    <span class=move || {
                        let count = char_count();
                        if show_counter && count > max_length {
                            "form-char-count form-char-count-over"
                        } else {
                            "form-char-count"
                        }
                    }>
                        {move || format!("{} / {}", char_count(), max_length)}
                    </span>
                </Show>
            </div>
            <textarea
                id=input_id
                class=move || if has_error() { "form-textarea form-textarea-error" } else { "form-textarea" }
                placeholder=placeholder
                prop:value=move || signal.get()
                on:input=move |ev| signal.set(event_target_value(&ev))
                aria-invalid=move || has_error().to_string()
            ></textarea>
            <Show when=has_error>
                <span class="form-error-msg" role="alert">
                    {move || error_msg().unwrap_or_default()}
                </span>
            </Show>
        </div>
    }
}
