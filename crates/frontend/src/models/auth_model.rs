// crates/frontend/src/models/auth_model.rs
// 목적: 인증 상태 전역 관리 래퍼.
// 로그인 상태, JWT 토큰, 현재 사용자 프로필을 Leptos RwSignal로 관리합니다.
// 컴포넌트가 is_authenticated Signal을 구독하여 로그인 여부에 따라 UI를 분기합니다.

use leptos::prelude::*;
use shared::dto::user_dto::{AuthTokenRes, UserProfileRes};

/// 전역 인증 상태 저장소
/// Context Provider로 앱 루트에서 provide_context()로 주입합니다.
#[derive(Clone, Debug)]
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
        let current_user = RwSignal::new(None::<UserProfileRes>);
        let access_token = RwSignal::new(None::<String>);

        let is_authenticated = Signal::derive(move || current_user.get().is_some());
        let is_admin = Signal::derive(move || {
            current_user
                .get()
                .map(|u| u.account_status == "admin") // role 기반으로 변경 필요
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
    pub fn login(&self, token_res: AuthTokenRes, profile: UserProfileRes) {
        self.access_token.set(Some(token_res.access_token));
        self.current_user.set(Some(profile));
        todo!("localStorage에 token 저장")
    }

    /// 로그아웃 — 상태 초기화 및 localStorage 제거
    pub fn logout(&self) {
        self.access_token.set(None);
        self.current_user.set(None);
        todo!("localStorage.removeItem('token')")
    }
}
