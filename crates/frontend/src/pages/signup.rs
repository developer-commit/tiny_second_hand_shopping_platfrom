// crates/frontend/src/pages/signup.rs
// URL: /signup

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;
use shared::dto::user_dto::SignUpReq;
use gloo_net::http::Request;
use crate::components::{
    layout::PageContainer,
    input::FormInput,
    feedback::{AppButton, ButtonVariant},
};

#[component]
pub fn SignupPage() -> impl IntoView {
    let account_id = RwSignal::new(String::new());
    let secret_key = RwSignal::new(String::new());
    let contact_email = RwSignal::new(String::new());
    let verification_code = RwSignal::new(String::new());
    let error_msg = RwSignal::new(Option::<String>::None);
    let success_msg = RwSignal::new(Option::<String>::None);

    let navigate = use_navigate();

    let signup_action = Action::new_local(move |req: &SignUpReq| {
        let req_clone = req.clone();
        async move {
            let res = Request::post("/v1/auth/signup")
                .json(&req_clone)
                .map_err(|e| e.to_string())?
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !res.ok() {
                let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
                let err_msg = err_res.map(|e| e.message).unwrap_or_else(|_| "Failed to signup".to_string());
                return Err(err_msg);
            }

            Ok(())
        }
    });

    let is_loading = signup_action.pending();

    let sendcode_action = Action::new_local(move |req: &shared::dto::user_dto::SendCodeReq| {
        let req_clone = req.clone();
        async move {
            let res = Request::post("/v1/auth/sendcode")
                .json(&req_clone)
                .map_err(|e| e.to_string())?
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !res.ok() {
                let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
                return Err(err_res.map(|e| e.message).unwrap_or_else(|_| "인증 코드 전송 실패".to_string()));
            }
            Ok(())
        }
    });

    let is_sending_code = sendcode_action.pending();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let req = SignUpReq {
            account_id: account_id.get(),
            secret_key: secret_key.get(),
            contact_email: Some(contact_email.get()),
            contact_phone: None,
            verification_code: verification_code.get(),
        };
        signup_action.dispatch(req);
    };

    Effect::new(move |_| {
        if let Some(res) = signup_action.value().get() {
            match res {
                Ok(_) => {
                    let nav = navigate.clone();
                    success_msg.set(Some("회원가입이 완료되었습니다. 로그인해주세요.".to_string()));
                    nav("/login", Default::default());
                }
                Err(e) => {
                    error_msg.set(Some(e));
                }
            }
        }
    });

    Effect::new(move |_| {
        if let Some(res) = sendcode_action.value().get() {
            match res {
                Ok(_) => success_msg.set(Some("인증 코드가 전송되었습니다.".to_string())),
                Err(e) => error_msg.set(Some(e)),
            }
        }
    });

    view! {
        <Title text="회원가입"/>
        <PageContainer title="회원가입">
            <div class="signup-page" style="max-width: 400px; margin: 0 auto; padding-top: 4rem;">
                <h1 style="text-align: center; font-size: 1.5rem; font-weight: bold; margin-bottom: 2rem;">"회원가입"</h1>
                <form on:submit=on_submit style="display: flex; flex-direction: column; gap: 1rem;">
                    <FormInput
                        label="아이디"
                        placeholder="아이디 (3~50자)"
                        input_type="text"
                        signal=account_id
                        error=Signal::derive(|| None)
                    />
                    <FormInput
                        label="비밀번호"
                        placeholder="비밀번호 (8자 이상)"
                        input_type="password"
                        signal=secret_key
                        error=Signal::derive(|| None)
                    />
                    <FormInput
                        label="이메일"
                        placeholder="이메일을 입력하세요"
                        input_type="email"
                        signal=contact_email
                        error=Signal::derive(|| None)
                    />
                    
                    <div style="display: flex; gap: 0.5rem; align-items: flex-end;">
                        <div style="flex: 1;">
                            <FormInput
                                label="인증코드"
                                placeholder="인증코드를 입력하세요"
                                input_type="text"
                                signal=verification_code
                                error=Signal::derive(|| None)
                            />
                        </div>
                        <AppButton
                            variant=ButtonVariant::Secondary
                            loading=is_sending_code
                            disabled=is_sending_code
                            on_click=move || {
                                let req = shared::dto::user_dto::SendCodeReq {
                                    account_id: account_id.get(),
                                    contact_email: Some(contact_email.get()),
                                    contact_phone: None,
                                };
                                sendcode_action.dispatch(req);
                            }
                        >
                            "코드 전송"
                        </AppButton>
                    </div>

                    {move || error_msg.get().map(|e| view! { <p class="error" style="color: red;">{e}</p> })}
                    {move || success_msg.get().map(|m| view! { <p class="success" style="color: green;">{m}</p> })}
                    
                    <AppButton
                        variant=ButtonVariant::Primary
                        loading=is_loading
                        disabled=is_loading
                        button_type="submit"
                    >
                        "가입하기"
                    </AppButton>
                </form>
            </div>
        </PageContainer>
    }
}
