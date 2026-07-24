// crates/frontend/src/models/auth_model.rs
// 목적: 인증 상태 전역 관리 래퍼.
// 로그인 상태, JWT 토큰, 현재 사용자 프로필을 Leptos RwSignal로 관리합니다.
// 컴포넌트가 is_authenticated Signal을 구독하여 로그인 여부에 따라 UI를 분기합니다.

use gloo_net::http::Request;
use leptos::prelude::*;
use shared::dto::user_dto::{AuthTokenRes, LoginReq, UserProfileRes};

const TOKEN_KEY: &str = "eth_access_token";
const API_BASE_URL: &str = "/v1";

/// localStorage 헬퍼 — WASM에서 web-sys 없이 JS eval 없이 안전하게 접근
fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// 전역 인증 상태 저장소
/// Context Provider로 앱 루트에서 provide_context()로 주입합니다.
#[derive(Clone, Copy, Debug)]
pub struct AuthStore {
    /// 현재 로그인된 사용자 프로필. None이면 비로그인 상태.
    pub current_user: RwSignal<Option<UserProfileRes>>,
    /// JWT 액세스 토큰 (localStorage와 동기화)
    pub access_token: RwSignal<Option<String>>,
    /// 로그인 상태 파생 시그널 (current_user가 Some인 경우)
    pub is_authenticated: Signal<bool>,
    /// 관리자 여부 파생 시그널
    pub is_admin: Signal<bool>,
}

impl AuthStore {
    pub fn new() -> Self {
        // localStorage에서 저장된 토큰 복원
        let saved_token = local_storage()
            .and_then(|s| s.get_item(TOKEN_KEY).ok())
            .flatten();

        let current_user = RwSignal::new(None::<UserProfileRes>);
        let access_token = RwSignal::new(saved_token);

        let is_authenticated = Signal::derive(move || current_user.get().is_some());
        let is_admin = Signal::derive(move || {
            current_user
                .get()
                .map(|u| u.role == "admin")
                .unwrap_or(false)
        });

        AuthStore {
            current_user,
            access_token,
            is_authenticated,
            is_admin,
        }
    }

    /// 로그인 성공 시 호출 — 토큰 저장 및 사용자 정보 설정
    pub fn set_login_state(&self, token_res: AuthTokenRes, profile: UserProfileRes) {
        // localStorage에 토큰 영속화
        if let Some(storage) = local_storage() {
            let _ = storage.set_item(TOKEN_KEY, &token_res.access_token);
        }
        self.access_token.set(Some(token_res.access_token));
        self.current_user.set(Some(profile));
    }

    /// 로그아웃 — 상태 초기화 및 localStorage 제거
    pub fn logout(&self) {
        if let Some(storage) = local_storage() {
            let _ = storage.remove_item(TOKEN_KEY);
        }
        self.access_token.set(None);
        self.current_user.set(None);
    }

    /// 현재 저장된 Bearer 토큰 헤더값 반환
    pub fn bearer_header(&self) -> Option<String> {
        self.access_token.get()
    }
}

/// 로그인 API Action (POST /auth/login & GET /users/me)
pub fn create_login_action(
    auth_store: AuthStore,
) -> Action<
    LoginReq,
    Result<shared::dto::user_dto::LoginResponse, String>,
    leptos::prelude::LocalStorage,
> {
    Action::new_local(move |req: &LoginReq| {
        let auth_store = auth_store.clone();
        let req_clone = req.clone();

        async move {
            // 1. POST /auth/login
            let login_res = Request::post(&format!("{}/auth/login", API_BASE_URL))
                .json(&req_clone)
                .map_err(|e| e.to_string())?
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !login_res.ok() {
                let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> =
                    login_res.json().await;
                let err_msg = err_res
                    .map(|e| e.message)
                    .unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
                return Err(err_msg);
            }

            let response_data: shared::dto::user_dto::LoginResponse =
                login_res.json().await.map_err(|e| e.to_string())?;

            match response_data {
                shared::dto::user_dto::LoginResponse::Success(token_res) => {
                    // 2. GET /users/me
                    let profile_res = Request::get(&format!("{}/users/me", API_BASE_URL))
                        .header(
                            "Authorization",
                            &format!("Bearer {}", token_res.access_token),
                        )
                        .send()
                        .await
                        .map_err(|e| e.to_string())?;

                    if !profile_res.ok() {
                        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> =
                            profile_res.json().await;
                        let err_msg = err_res
                            .map(|e| e.message)
                            .unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
                        return Err(err_msg);
                    }

                    let profile: UserProfileRes =
                        profile_res.json().await.map_err(|e| e.to_string())?;

                    // 3. Update State
                    auth_store.set_login_state(token_res.clone(), profile);

                    Ok(shared::dto::user_dto::LoginResponse::Success(token_res))
                }
                shared::dto::user_dto::LoginResponse::Requires2FA { user_uid } => {
                    Ok(shared::dto::user_dto::LoginResponse::Requires2FA { user_uid })
                }
            }
        }
    })
}
