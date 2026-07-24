// crates/frontend/src/components/feedback/button.rs
// 목적: 공용 버튼 — variant별 스타일, 로딩 스피너, 비활성화 처리

use leptos::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
    Ghost,
}

impl ButtonVariant {
    fn class(&self) -> &'static str {
        match self {
            ButtonVariant::Primary => "btn btn-primary",
            ButtonVariant::Secondary => "btn btn-secondary",
            ButtonVariant::Danger => "btn btn-danger",
            ButtonVariant::Ghost => "btn btn-ghost",
        }
    }
}

#[component]
pub fn AppButton(
    #[prop(default = ButtonVariant::Primary)] variant: ButtonVariant,
    /// 로딩 중 여부 (true이면 스피너 표시, 클릭 불가)
    #[prop(optional, into)]
    loading: Option<Signal<bool>>,
    /// 비활성화 여부
    #[prop(optional, into)]
    disabled: Option<Signal<bool>>,
    /// 클릭 핸들러
    #[prop(optional, into)]
    on_click: Option<Callback<()>>,
    /// button type (submit / button / reset)
    #[prop(into, default = "button".to_string())]
    button_type: String,
    /// 자식 콘텐츠
    children: Children,
) -> impl IntoView {
    let is_loading = move || loading.map(|s| s.get()).unwrap_or(false);
    let is_disabled = move || disabled.map(|s| s.get()).unwrap_or(false) || is_loading();

    let base_class = variant.class();
    let class = move || {
        if is_loading() {
            format!("{} btn-loading", base_class)
        } else {
            base_class.to_string()
        }
    };

    view! {
        <button
            type=button_type
            class=class
            disabled=move || is_disabled()
            on:click=move |_| {
                if !is_disabled() {
                    if let Some(cb) = on_click {
                        cb.run(());
                    }
                }
            }
            aria-busy=move || is_loading().to_string()
        >
            <Show
                when=is_loading
                fallback=|| ()
            >
                <span class="btn-spinner" aria-hidden="true"></span>
            </Show>
            {children()}
        </button>
    }
}
