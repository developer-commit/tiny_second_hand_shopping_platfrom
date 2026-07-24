// crates/frontend/src/components/feedback/toast.rs
// 목적: 전역 토스트 알림 시스템
// ToastStore: Context로 주입되는 전역 알림 큐
// ToastContainer: 화면 우하단에 고정 렌더링되는 알림 목록

use leptos::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

static TOAST_ID_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Debug, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub id: u32,
    pub kind: ToastKind,
    pub message: String,
}

/// 전역 토스트 알림 저장소 — provide_context로 앱 루트에 주입합니다.
#[derive(Clone, Debug)]
pub struct ToastStore {
    pub toasts: RwSignal<Vec<Toast>>,
}

impl ToastStore {
    pub fn new() -> Self {
        ToastStore {
            toasts: RwSignal::new(Vec::new()),
        }
    }

    /// 토스트 추가 — 3초 후 자동 제거
    pub fn push(&self, kind: ToastKind, message: impl Into<String>) {
        let id = TOAST_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let toast = Toast { id, kind, message: message.into() };
        self.toasts.update(|v| v.push(toast));

        // 3초 후 자동 제거 (JS setTimeout)
        let toasts = self.toasts;
        let remove_id = id;
        let closure = wasm_bindgen::closure::Closure::<dyn Fn()>::new(move || {
            toasts.update(|v| v.retain(|t| t.id != remove_id));
        });
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                wasm_bindgen::JsCast::unchecked_ref(closure.as_ref()),
                3000,
            );
        }
        closure.forget();
    }

    pub fn success(&self, msg: impl Into<String>) { self.push(ToastKind::Success, msg); }
    pub fn error(&self, msg: impl Into<String>)   { self.push(ToastKind::Error, msg); }
    pub fn warning(&self, msg: impl Into<String>) { self.push(ToastKind::Warning, msg); }
    pub fn info(&self, msg: impl Into<String>)    { self.push(ToastKind::Info, msg); }

    /// 특정 토스트 수동 제거
    pub fn dismiss(&self, id: u32) {
        self.toasts.update(|v| v.retain(|t| t.id != id));
    }
}

/// 화면 우하단 고정 토스트 컨테이너 — main.rs 루트에 한 번만 배치합니다.
#[component]
pub fn ToastContainer() -> impl IntoView {
    let store = use_context::<ToastStore>()
        .expect("ToastStore must be provided");

    view! {
        <div class="toast-container" aria-live="polite" aria-atomic="false">
            {move || store.toasts.get().into_iter().map(|toast| {
                let dismiss_id = toast.id;
                let store_clone = store.clone();
                let toast_class = match toast.kind {
                    ToastKind::Success => "toast toast-success",
                    ToastKind::Error   => "toast toast-error",
                    ToastKind::Warning => "toast toast-warning",
                    ToastKind::Info    => "toast toast-info",
                };
                let icon = match toast.kind {
                    ToastKind::Success => "✅",
                    ToastKind::Error   => "❌",
                    ToastKind::Warning => "⚠️",
                    ToastKind::Info    => "ℹ️",
                };

                view! {
                    <div class=toast_class role="alert">
                        <span class="toast-icon">{icon}</span>
                        <span class="toast-message">{toast.message.clone()}</span>
                        <button
                            type="button"
                            class="toast-dismiss"
                            on:click=move |_| store_clone.dismiss(dismiss_id)
                            aria-label="알림 닫기"
                        >
                            "✕"
                        </button>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
