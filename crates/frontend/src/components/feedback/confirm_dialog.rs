// crates/frontend/src/components/feedback/confirm_dialog.rs
// 목적: 확인/취소 다이얼로그 — Modal 래핑, 단순 yes/no 인터랙션

use leptos::prelude::*;
use super::Modal;
use super::AppButton;
use super::ButtonVariant;

#[component]
pub fn ConfirmDialog(
    is_open: RwSignal<bool>,
    #[prop(into)]
    message: String,
    on_confirm: Callback<()>,
    on_cancel: Callback<()>,
    /// 확인 버튼 레이블 (기본: "확인")
    #[prop(into, default = "확인".to_string())]
    confirm_label: String,
    /// 확인 버튼 variant (기본: Danger)
    #[prop(default = ButtonVariant::Danger)]
    confirm_variant: ButtonVariant,
) -> impl IntoView {
    let on_confirm_click = move |_: ()| {
        is_open.set(false);
        on_confirm.run(());
    };
    let on_cancel_click = move |_: ()| {
        is_open.set(false);
        on_cancel.run(());
    };

    let message = StoredValue::new(message);
    let confirm_label = StoredValue::new(confirm_label);

    view! {
        <Modal is_open=is_open title="확인">
            <p class="confirm-message">{move || message.get_value()}</p>
            <div class="confirm-actions">
                <AppButton
                    variant=ButtonVariant::Ghost
                    on_click=Callback::new(on_cancel_click)
                >
                    "취소"
                </AppButton>
                <AppButton
                    variant=confirm_variant
                    on_click=Callback::new(on_confirm_click)
                >
                    {move || confirm_label.get_value()}
                </AppButton>
            </div>
        </Modal>
    }
}
