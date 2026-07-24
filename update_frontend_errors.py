import os
import glob
import re

model_dir = "crates/frontend/src/models"
files = glob.glob(f"{model_dir}/*.rs")

for f in files:
    with open(f, "r") as file:
        content = file.read()
    
    # Matches if !res.ok() { ... }
    def replacer(match):
        res_var = match.group(1)
        return f"""if !{res_var}.ok() {{
        let err_res: Result<shared::dto::error_dto::ApiErrorRes, _> = {res_var}.json().await;
        let err_msg = err_res.map(|e| e.message).unwrap_or_else(|_| "오류가 발생했습니다.".to_string());
        return Err(err_msg);
    }}"""

    # We match: if !res.ok() { any characters until the closing brace }
    content = re.sub(r'if !([a-zA-Z0-9_]+)\.ok\(\)\s*\{[^}]*\}', replacer, content)
    
    with open(f, "w") as file:
        file.write(content)

print("Updated frontend models")
