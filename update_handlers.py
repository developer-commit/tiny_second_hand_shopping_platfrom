import os
import glob
import re

handler_dir = "crates/backend/src/handlers"
files = glob.glob(f"{handler_dir}/*.rs")

for f in files:
    with open(f, "r") as file:
        content = file.read()
    
    # Add use crate::utils::error::AppError;
    if "use crate::utils::error::AppError;" not in content:
        content = content.replace("use axum::{", "use crate::utils::error::AppError;\nuse axum::{")
    
    # Replace Result<..., StatusCode> with Result<..., AppError>
    content = content.replace("Result<StatusCode, StatusCode>", "Result<StatusCode, AppError>")
    content = re.sub(r"Result<(.*?), StatusCode>", r"Result<\1, AppError>", content)
    
    # Replace map_err(|e| { ... StatusCode::... }) with AppError mapping
    # This might be tricky via simple regex, but let's try a simple sed-like approach
    # Just replacing StatusCode::UNAUTHORIZED with AppError::Unauthorized
    content = content.replace("StatusCode::UNPROCESSABLE_ENTITY", "AppError::BadRequest(\"Invalid request\".to_string())")
    content = content.replace("StatusCode::UNAUTHORIZED", "AppError::Unauthorized")
    content = content.replace("StatusCode::INTERNAL_SERVER_ERROR", "AppError::Internal")
    content = content.replace("StatusCode::NOT_FOUND", "AppError::NotFound(\"Not found\".to_string())")
    
    with open(f, "w") as file:
        file.write(content)

print("Updated handlers")
