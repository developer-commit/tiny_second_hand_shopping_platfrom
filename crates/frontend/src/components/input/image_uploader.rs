// crates/frontend/src/components/input/image_uploader.rs
// 목적: 드래그 앤 드롭 / 파일 선택 이미지 업로드 UI
// Phase 1: UI 구조 구현 (on_uploaded 콜백은 Phase 2 API 연동 시 활성화)

use leptos::prelude::*;
use crate::models::auth_model::AuthStore;
use gloo_net::http::Request;

#[derive(serde::Deserialize)]
struct UploadResponse {
    image_url: String,
}

#[component]
pub fn ImageUploader(
    /// 업로드 완료된 이미지 URL을 부모에게 전달하는 콜백
    on_uploaded: Callback<String>,
    /// 현재 미리보기 URL 목록
    preview_urls: RwSignal<Vec<String>>,
    /// 최대 업로드 개수
    #[prop(default = 5_usize)]
    max_count: usize,
) -> impl IntoView {
    let can_add = move || preview_urls.get().len() < max_count;
    let is_uploading = RwSignal::new(false);
    
    let auth_store = expect_context::<AuthStore>();
    let token = Signal::derive(move || auth_store.bearer_header().unwrap_or_default());

    let on_file_change = move |ev: leptos::ev::Event| {
        use wasm_bindgen::JsCast;
        let input = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());

        if let Some(input) = input {
            if let Some(files) = input.files() {
                for i in 0..files.length() {
                    if let Some(file) = files.item(i) {
                        let t = token.get();
                        let on_uploaded_cb = on_uploaded.clone();
                        
                        is_uploading.set(true);
                        
                        leptos::task::spawn_local(async move {
                            if t.is_empty() {
                                leptos::logging::error!("No auth token available for upload");
                                is_uploading.set(false);
                                return;
                            }

                            let form_data = web_sys::FormData::new().unwrap();
                            let _ = form_data.append_with_blob("file", &file);

                            let req = match Request::post("/v1/uploads")
                                .header("Authorization", &format!("Bearer {}", t))
                                .body(form_data)
                            {
                                Ok(r) => r,
                                Err(e) => {
                                    leptos::logging::error!("Request builder error: {}", e);
                                    is_uploading.set(false);
                                    return;
                                }
                            };

                            let res = req.send().await;
                            is_uploading.set(false);

                            match res {
                                Ok(response) if response.ok() => {
                                    if let Ok(upload_res) = response.json::<UploadResponse>().await {
                                        preview_urls.update(|v| v.push(upload_res.image_url.clone()));
                                        on_uploaded_cb.run(upload_res.image_url);
                                    }
                                }
                                Ok(response) => leptos::logging::error!("Upload failed: {}", response.status()),
                                Err(e) => leptos::logging::error!("Upload error: {:?}", e),
                            }
                        });
                    }
                }
            }
            input.set_value("");
        }
    };

    let on_remove = move |idx: usize| {
        preview_urls.update(|v| { v.remove(idx); });
    };

    view! {
        <div class="image-uploader">
            <div class="image-uploader-grid">
                // 미리보기 썸네일들
                {move || preview_urls.get().into_iter().enumerate().map(|(idx, url)| {
                    view! {
                        <div class="image-preview-item">
                            <img src=url.clone() alt=format!("이미지 {}", idx + 1) class="image-preview-thumb"/>
                            <button
                                type="button"
                                class="image-preview-remove"
                                on:click=move |_| on_remove(idx)
                                aria-label=format!("이미지 {} 삭제", idx + 1)
                            >
                                "×"
                            </button>
                        </div>
                    }
                }).collect_view()}

                // 추가 버튼
                <Show when=can_add>
                    <label class="image-uploader-add" aria-label="이미지 추가">
                        <span class="image-uploader-add-icon">
                            {move || if is_uploading.get() { "⏳" } else { "📷" }}
                        </span>
                        <span class="image-uploader-add-text">
                            {move || if is_uploading.get() { "업로드 중..." } else { "사진 추가" }}
                        </span>
                        <span class="image-uploader-add-count">
                            {move || format!("{}/{}", preview_urls.get().len(), max_count)}
                        </span>
                        <input
                            type="file"
                            accept="image/*"
                            multiple
                            class="image-uploader-input"
                            on:change=on_file_change
                            disabled=move || is_uploading.get()
                        />
                    </label>
                </Show>
            </div>
        </div>
    }
}
