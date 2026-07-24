// crates/frontend/src/pages/login.rs
// URL: /login
// 목적: 로그인 페이지

use crate::components::{
    feedback::{AppButton, ButtonVariant},
    input::FormInput,
    layout::PageContainer,
};
use crate::models::auth_model::{AuthStore, create_login_action};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;
use shared::dto::user_dto::LoginReq;

#[component]
pub fn LoginPage() -> impl IntoView {
    let account_id = RwSignal::new(String::new());
    let secret_key = RwSignal::new(String::new());
    let error_msg = RwSignal::new(Option::<String>::None);
    let requires_2fa = RwSignal::new(false);
    let otp_token = RwSignal::new(String::new());

    let navigate = use_navigate();
    let auth_store = expect_context::<AuthStore>();

    let login_action = create_login_action(auth_store);
    let is_loading = login_action.pending();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let req = LoginReq {
            account_id: account_id.get(),
            secret_key: secret_key.get(),
        };
        login_action.dispatch(req);
    };

    Effect::new(move |_| {
        if let Some(res) = login_action.value().get() {
            match res {
                Ok(shared::dto::user_dto::LoginResponse::Success(_)) => {
                    navigate("/", Default::default());
                }
                Ok(shared::dto::user_dto::LoginResponse::Requires2FA { .. }) => {
                    requires_2fa.set(true);
                }
                Err(e) => {
                    error_msg.set(Some(e));
                }
            }
        }
    });

    view! {
        <Title text="로그인"/>
        <PageContainer title="로그인">
            <div class="login-page" style="max-width: 400px; margin: 0 auto; padding-top: 4rem;">
                <Show
                    when=move || requires_2fa.get()
                    fallback=move || view! {
                        <h1 style="text-align: center; font-size: 1.5rem; font-weight: bold; margin-bottom: 2rem;">"로그인"</h1>
                        <form on:submit=on_submit style="display: flex; flex-direction: column; gap: 1rem;">
                            <FormInput
                                label="아이디"
                                placeholder="아이디를 입력하세요"
                                input_type="text"
                                signal=account_id
                                error=Signal::derive(|| None)
                            />
                            <FormInput
                                label="비밀번호"
                                placeholder="비밀번호를 입력하세요"
                                input_type="password"
                                signal=secret_key
                                error=Signal::derive(|| None)
                            />
                            {move || error_msg.get().map(|e| view! { <p class="error" style="color: red;">{e}</p> })}

                            <AppButton
                                variant=ButtonVariant::Primary
                                loading=is_loading
                                disabled=is_loading
                                button_type="submit"
                            >
                                "로그인"
                            </AppButton>
                        </form>
                        <div style="text-align: center; margin-top: 1.5rem;">
                            <a href="/signup" style="color: var(--primary-color); text-decoration: none;">"계정이 없으신가요? 회원가입"</a>
                        </div>
                    }
                >
                    <h1 style="text-align: center; font-size: 1.5rem; font-weight: bold; margin-bottom: 2rem;">"2단계 인증"</h1>
                    <form style="display: flex; flex-direction: column; gap: 1rem;">
                        <FormInput
                            label="OTP 코드"
                            placeholder="6자리 코드 입력"
                            input_type="text"
                            signal=otp_token
                            error=Signal::derive(|| None)
                        />
                        <AppButton variant=ButtonVariant::Primary loading=Signal::derive(|| false) disabled=Signal::derive(|| false) button_type="submit">
                            "인증 및 로그인"
                        </AppButton>
                    </form>
                </Show>
            </div>
        </PageContainer>
    }
}
