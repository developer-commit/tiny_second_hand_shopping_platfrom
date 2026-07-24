// crates/frontend/src/pages/mypage.rs
// URL: /mypage

use leptos::prelude::*;
use leptos_meta::Title;
use crate::components::{
    layout::PageContainer,
    display::UserTrustIndicator,
    input::OtpInput,
    feedback::{AppButton, ButtonVariant, Modal, QrCodeDisplay},
};
use crate::models::auth_model::AuthStore;
use shared::dto::user_dto::{TwoFaSetupRes, Enable2FaReq};
use gloo_net::http::Request;

async fn setup_2fa(token: &str) -> Result<TwoFaSetupRes, String> {
    let url = format!("{}/users/me/2fa/setup", "/v1");
    let res = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        return Err("2FA 설정 시작 실패".into());
    }
    res.json().await.map_err(|e| e.to_string())
}

async fn enable_2fa(token: &str, req: &Enable2FaReq) -> Result<(), String> {
    let url = format!("{}/users/me/2fa/enable", "/v1");
    let res = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        return Err("2FA 활성화 실패".into());
    }
    Ok(())
}

#[component]
pub fn MyPage() -> impl IntoView {
    let show_2fa_modal = RwSignal::new(false);
    let otp_input = RwSignal::new(String::new());
    
    let auth_store = expect_context::<AuthStore>();
    let current_user = auth_store.current_user;
    let token = Signal::derive(move || auth_store.bearer_header().unwrap_or_default());

    let setup_res = LocalResource::new(move || {
        let is_open = show_2fa_modal.get();
        let t = token.get();
        async move {
            if !is_open || t.is_empty() { return Err("미실행".to_string()); }
            setup_2fa(&t).await
        }
    });

    let enable_action = Action::new_local(move |req: &Enable2FaReq| {
        let t = token.get();
        let req_clone = req.clone();
        async move {
            enable_2fa(&t, &req_clone).await
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(_)) = enable_action.value().get() {
            show_2fa_modal.set(false);
            // Could refresh user profile to show 2FA is active
        }
    });

    view! {
        <Title text="마이페이지"/>
        <PageContainer title="마이페이지">
            <div class="mypage" style="max-width: 600px; margin: 0 auto; padding-top: 2rem;">
                <h1 style="font-size: 1.5rem; font-weight: bold; margin-bottom: 2rem;">"마이페이지"</h1>
                
                <section class="profile-section" style="background: var(--surface-color); padding: 1.5rem; border-radius: 8px;">
                    {move || match current_user.get() {
                        Some(user) => view! {
                            <div style="display: flex; flex-direction: column; gap: 1rem;">
                                <div>
                                    <h2 style="font-size: 1.25rem; font-weight: 600;">{user.display_name}</h2>
                                    <p style="color: var(--text-secondary);">{user.contact_email}</p>
                                </div>
                                <UserTrustIndicator reliability_index=user.reliability_index />
                                <div>
                                    <span style="font-weight: bold;">"가입일: "</span>
                                    <span>{user.joined_at}</span>
                                </div>
                            </div>
                        }.into_any(),
                        None => view! { <p>"프로필 정보를 불러올 수 없습니다."</p> }.into_any(),
                    }}
                </section>

                <section class="security-section">
                    <h2 style="font-size: 1.25rem; font-weight: bold; margin-bottom: 1rem;">"보안 설정"</h2>
                    <div style="display: flex; gap: 1rem; align-items: center;">
                        <AppButton
                            variant=ButtonVariant::Secondary
                            loading=Signal::derive(|| false)
                            disabled=Signal::derive(|| false)
                            on_click=move || show_2fa_modal.set(true)
                        >
                            "2FA 설정"
                        </AppButton>
                    </div>
                </section>
                
                // 2FA 설정 모달
                <Modal is_open=show_2fa_modal title="2FA 설정">
                    <div style="display: flex; flex-direction: column; gap: 1.5rem; padding-top: 1rem; align-items: center;">
                        <p style="text-align: center;">
                            "인증 앱(Google Authenticator 등)으로 아래 QR 코드를 스캔하세요."
                        </p>
                        
                        <Suspense fallback=move || view! { <p>"QR 코드 생성 중..."</p> }>
                            {move || setup_res.get().map(|res| match &*res {
                                Ok(setup) => view! {
                                    <QrCodeDisplay data=setup.qr_code_url.clone() />
                                    <p style="font-size: 0.8rem; color: var(--text-secondary);">
                                        "수동 입력 키: " {setup.manual_entry_key.clone()}
                                    </p>
                                }.into_any(),
                                Err(e) if e == "미실행" => view! { <span></span> }.into_any(),
                                Err(e) => view! { <p style="color: red;">{e.clone()}</p> }.into_any(),
                            })}
                        </Suspense>
                        
                        <div style="width: 100%;">
                            <label style="display: block; margin-bottom: 0.5rem; font-size: 0.875rem;">"인증 코드 (6자리)"</label>
                            <OtpInput length=6 signal=otp_input />
                        </div>
                        
                        <div style="display: flex; gap: 1rem; width: 100%;">
                            <div style="flex: 1;">
                                <AppButton
                                    variant=ButtonVariant::Ghost
                                    loading=Signal::derive(|| false)
                                    disabled=Signal::derive(|| false)
                                    on_click=move || show_2fa_modal.set(false)
                                >
                                    "취소"
                                </AppButton>
                            </div>
                            <div style="flex: 1;">
                                <AppButton
                                    variant=ButtonVariant::Primary
                                    loading=enable_action.pending()
                                    disabled=Signal::derive(move || enable_action.pending().get() || otp_input.get().len() < 6)
                                    on_click=move || {
                                        enable_action.dispatch(Enable2FaReq {
                                            otp_token: otp_input.get(),
                                        });
                                    }
                                >
                                    "확인"
                                </AppButton>
                            </div>
                        </div>
                    </div>
                </Modal>
            </div>
        </PageContainer>
    }
}
