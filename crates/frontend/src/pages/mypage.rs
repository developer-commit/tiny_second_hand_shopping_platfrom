// crates/frontend/src/pages/mypage.rs
// URL: /mypage

use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn MyPage() -> impl IntoView {
    let show_2fa_modal = RwSignal::new(false);
    let otp_input = RwSignal::new(String::new());

    view! {
        <Title text="마이페이지"/>
        <main class="mypage">
            <h1>"마이페이지"</h1>
            <section class="profile-section">
                // todo!("AuthStore에서 current_user 조회 → 프로필 표시")
                <p>"프로필 정보"</p>
            </section>
            <section class="security-section">
                <h2>"보안 설정"</h2>
                <button on:click=move |_| show_2fa_modal.set(true)>"2FA 설정"</button>
            </section>
            // 2FA 설정 모달
            <Show when=move || show_2fa_modal.get()>
                <div class="modal">
                    <h3>"2FA 설정"</h3>
                    // todo!("QR 코드 표시 + OTP 입력 → /v1/users/me/2fa/enable 호출")
                    <input type="text" placeholder="OTP 코드" on:input=move |e| otp_input.set(event_target_value(&e)) />
                    <button on:click=move |_| show_2fa_modal.set(false)>"취소"</button>
                </div>
            </Show>
        </main>
    }
}
