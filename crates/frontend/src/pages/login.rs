// crates/frontend/src/pages/login.rs
// URL: /login
// 목적: 로그인 페이지

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;
use shared::dto::user_dto::LoginReq;

#[component]
pub fn LoginPage() -> impl IntoView {
    let account_id = RwSignal::new(String::new());
    let secret_key = RwSignal::new(String::new());
    let error_msg = RwSignal::new(Option::<String>::None);

    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let req = LoginReq {
            account_id: account_id.get(),
            secret_key: secret_key.get(),
        };
        todo!("POST /v1/auth/login → AuthStore.login() → navigate('/')")
    };

    view! {
        <Title text="로그인"/>
        <main class="login-page">
            <h1>"로그인"</h1>
            <form on:submit=on_submit>
                <input
                    type="text"
                    placeholder="아이디"
                    on:input=move |e| account_id.set(event_target_value(&e))
                />
                <input
                    type="password"
                    placeholder="비밀번호"
                    on:input=move |e| secret_key.set(event_target_value(&e))
                />
                {move || error_msg.get().map(|e| view! { <p class="error">{e}</p> })}
                <button type="submit">"로그인"</button>
            </form>
            <a href="/signup">"회원가입"</a>
        </main>
    }
}
