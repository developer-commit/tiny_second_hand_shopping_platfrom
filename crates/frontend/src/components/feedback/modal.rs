// crates/frontend/src/components/feedback/modal.rs
// 목적: 오버레이 모달 — ESC 키 / 배경 클릭으로 닫기
// is_open RwSignal을 통해 외부에서 제어합니다.
// <Show> 내부에서 children()을 호출하면 FnOnce 문제가 생기므로
// 조건부 렌더링에 portal 패턴 대신 is_open 기반 CSS class + 직접 렌더링을 사용합니다.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

#[component]
pub fn Modal(
    is_open: RwSignal<bool>,
    #[prop(into)] title: String,
    children: Children,
) -> impl IntoView {
    // ESC 키로 닫기 — document keydown 이벤트 등록
    Effect::new(move |_| {
        if is_open.get() {
            let closure = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
                move |ev: web_sys::KeyboardEvent| {
                    if ev.key() == "Escape" {
                        is_open.set(false);
                    }
                },
            );

            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                let _ = document
                    .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
                closure.forget();
            }
        }
    });

    let title = StoredValue::new(title);

    // children()을 미리 호출하여 AnyView로 고정
    let body = children();

    view! {
        // 오버레이 + 모달 박스를 항상 DOM에 유지하되, hidden 클래스로 가시성 제어
        <div
            class=move || if is_open.get() { "modal-overlay" } else { "modal-overlay hidden" }
            on:click=move |_| is_open.set(false)
            aria-hidden="true"
        ></div>
        <div
            class=move || if is_open.get() { "modal" } else { "modal hidden" }
            role="dialog"
            aria-modal=move || is_open.get().to_string()
            aria-labelledby="modal-title"
        >
            <div class="modal-header">
                <h2 id="modal-title" class="modal-title">
                    {move || title.get_value()}
                </h2>
                <button
                    type="button"
                    class="modal-close"
                    on:click=move |_| is_open.set(false)
                    aria-label="모달 닫기"
                >
                    "✕"
                </button>
            </div>
            <div class="modal-body">
                {body}
            </div>
        </div>
    }
}
