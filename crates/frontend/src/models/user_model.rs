// crates/frontend/src/models/user_model.rs

use gloo_net::http::Request;
use shared::dto::user_dto::PublicUserProfileRes;

const API_BASE_URL: &str = "/v1";

/// GET /users/{user_uid} - 공개 프로필 조회 API
pub async fn fetch_public_profile(user_uid: String) -> Result<PublicUserProfileRes, String> {
    let url = format!("{}/users/{}", API_BASE_URL, user_uid);
    let res = Request::get(&url).send().await.map_err(|e| e.to_string())?;

    if !res.ok() {
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = res.json().await;
        let err_msg = err_res
            .map(|e| e.message)
            .unwrap_or_else(|_| "프로필을 불러올 수 없습니다.".to_string());
        return Err(err_msg);
    }

    res.json().await.map_err(|e| e.to_string())
}
