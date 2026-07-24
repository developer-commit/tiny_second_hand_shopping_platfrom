use crate::utils::error::AppError;
use axum::{
    extract::Multipart,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use std::path::Path;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use tracing::{info, error};

#[derive(Serialize)]
pub struct UploadResponse {
    pub image_url: String,
}

// 5MB limit
const MAX_UPLOAD_SIZE: usize = 5 * 1024 * 1024;

pub async fn upload_image(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut uploaded_filename = String::new();
    
    let upload_dir = Path::new("uploads");
    // Removed TOCTOU vulnerability here, create_dir_all is idempotent
    tokio::fs::create_dir_all(upload_dir).await.map_err(|e| {
        error!("Failed to create uploads directory: {}", e);
        AppError::Internal
    })?;

    while let Some(mut field) = multipart.next_field().await.map_err(|e| {
        error!("Multipart parsing error: {}", e);
        AppError::BadRequest("잘못된 요청 형식입니다.".to_string())
    })? {
        let content_type = field.content_type().unwrap_or("").to_string();
        
        if !content_type.starts_with("image/") {
            return Err(AppError::BadRequest("이미지 파일만 업로드 가능합니다.".to_string()));
        }

        let extension = match content_type.as_str() {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            _ => return Err(AppError::BadRequest("지원하지 않는 이미지 형식입니다.".to_string())),
        };

        let filename = format!("{}.{}", Uuid::new_v4(), extension);
        let filepath = upload_dir.join(&filename);

        // Security: Prevent File Overwrite using create_new
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&filepath)
            .await
            .map_err(|e| {
                error!("Failed to create file: {}", e);
                AppError::Internal
            })?;

        let mut total_size = 0;
        let mut is_first_chunk = true;

        // Stream the file instead of loading it entirely in memory
        while let Some(chunk) = field.chunk().await.map_err(|e| {
            error!("Failed to read field chunk: {}", e);
            AppError::Internal
        })? {
            // Security: Content-Type Spoofing Mitigation (Magic bytes check on first chunk)
            if is_first_chunk {
                if chunk.len() > 0 {
                    let kind = infer::get(&chunk);
                    let valid_mime = kind.map(|k| k.mime_type()).unwrap_or("");
                    if !valid_mime.starts_with("image/") {
                        // Delete the partially created file
                        let _ = tokio::fs::remove_file(&filepath).await;
                        return Err(AppError::BadRequest("위조된 파일 형식입니다.".to_string()));
                    }
                }
                is_first_chunk = false;
            }

            total_size += chunk.len();
            if total_size > MAX_UPLOAD_SIZE {
                // Delete the file and return error
                let _ = tokio::fs::remove_file(&filepath).await;
                return Err(AppError::BadRequest("파일 크기는 5MB를 초과할 수 없습니다.".to_string()));
            }

            file.write_all(&chunk).await.map_err(|e| {
                error!("Failed to write to file: {}", e);
                AppError::Internal
            })?;
        }

        uploaded_filename = filename;
        break; 
    }

    if uploaded_filename.is_empty() {
        return Err(AppError::BadRequest("업로드된 파일이 없습니다.".to_string()));
    }

    let image_url = format!("http://localhost:8080/uploads/{}", uploaded_filename);
    info!("Image uploaded successfully: {}", image_url);

    Ok((StatusCode::CREATED, Json(UploadResponse { image_url })))
}
