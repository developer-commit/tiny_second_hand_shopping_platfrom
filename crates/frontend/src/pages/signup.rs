// crates/frontend/src/pages/signup.rs
// URL: /signup

use leptos::prelude::*;
use leptos_meta::Title;
use shared::dto::user_dto::SignUpReq;

#[component]
pub fn SignupPage() -> impl IntoView {
    let account_id = RwSignal::new(String::new());
    let secret_key = RwSignal::new(String::new());
    let contact_email = RwSignal::new(String::new());
    let verification_code = RwSignal::new(String::new());
    let error_msg = RwSignal::new(Option::<String>::None);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let req = SignUpReq {
            account_id: account_id.get(),
            secret_key: secret_key.get(),
            contact_email: Some(contact_email.get()),
            contact_phone: None,
            verification_code: verification_code.get(),
        };
        todo!("POST /v1/auth/signup → 성공 시 /login으로 이동")
    };

    view! {
        <Title text="회원가입"/>
        <main class="signup-page">
            <h1>"회원가입"</h1>
            <form on:submit=on_submit>
                <input type="text" placeholder="아이디 (3~50자)" on:input=move |e| account_id.set(event_target_value(&e)) />
                <input type="password" placeholder="비밀번호 (8자 이상)" on:input=move |e| secret_key.set(event_target_value(&e)) />
                <input type="email" placeholder="이메일" on:input=move |e| contact_email.set(event_target_value(&e)) />
                <button type="button">{"인증코드 전송"}</button>
                <input type="text" placeholder="인증코드" on:input=move |e| verification_code.set(event_target_value(&e)) />
                {move || error_msg.get().map(|e| view! { <p class="error">{e}</p> })}
                <button type="submit">"가입하기"</button>
            </form>
        </main>
    }
}
