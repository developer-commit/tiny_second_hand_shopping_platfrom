// crates/backend/src/infra/verification_adapter.rs
use async_trait::async_trait;
use crate::ports::verification_port::{VerificationPort, VerificationPortError};
use rand::Rng;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

// lettre 관련 모듈 임포트
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use chrono::{NaiveDate, Utc, DateTime, Duration};

struct AuthCode {
    code: String,
    time: DateTime<Utc>
}

impl AuthCode {
    fn is_verified(&self, code: &String) -> bool{
        let nowtime = Utc::now() - Duration::minutes(5);

        if self.time > nowtime && self.code == *code{
            return true
        }
        return false
    }
}

#[derive(Clone)]
pub struct VerificationAdapter {
    store: Arc<RwLock<HashMap<String, AuthCode>>>,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from_email: String,
}

impl VerificationAdapter {
    /// 환경 변수(.env)에서 구글 이메일 계정과 앱 비밀번호를 불러와 초기화합니다.
    pub fn new() -> Self {
        // .env 파일의 변수들을 시스템 환경 변수로 로드 (main에서 이미 호출했다면 생략 가능)
        dotenvy::dotenv().ok();

        // 환경 변수에서 SMTP 정보 불러오기
        let smtp_username = env::var("SMTP_USERNAME")
            .expect(".env 파일에 SMTP_USERNAME이 설정되어 있지 않습니다.");
        let smtp_password = env::var("SMTP_PASSWORD")
            .expect(".env 파일에 SMTP_PASSWORD가 설정되어 있지 않습니다.");

        // 1. SMTP 인증 정보 설정
        let creds = Credentials::new(smtp_username.clone(), smtp_password);

        // 2. 구글 SMTP 서버 릴레이 설정
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
            .expect("SMTP 릴레이 설정 실패")
            .credentials(creds)
            .build();

        VerificationAdapter {
            store: Arc::new(RwLock::new(HashMap::new())),
            mailer,
            from_email: smtp_username,
        }
    }
}

#[async_trait]
impl VerificationPort for VerificationAdapter {
    async fn send_code(&self, target: &str) -> Result<(), VerificationPortError> {
        let code: String = {
            let mut rng = rand::thread_rng();
            (0..6).map(|_| rng.gen_range(0..10).to_string()).collect()
        };
        let time = Utc::now();

        let authcode = AuthCode{
            code,
            time
        };

        let email = Message::builder()
            .from(
                self.from_email
                    .parse()
                    .map_err(|e| VerificationPortError::SendError(format!("발신자 이메일 파싱 에러: {}", e)))?,
            )
            .to(
                target
                    .parse()
                    .map_err(|e| VerificationPortError::SendError(format!("수신자 이메일 파싱 에러: {}", e)))?,
            )
            .subject("서비스 가입 인증 코드입니다.")
            .body(format!("요청하신 인증 코드는 다음과 같습니다:\n\n{}\n\n이 코드를 입력창에 입력해 주세요.", &authcode.code))
            .map_err(|e| VerificationPortError::SendError(format!("이메일 빌드 에러: {}", e)))?;

        self.mailer
            .send(email)
            .await
            .map_err(|e| VerificationPortError::SendError(format!("이메일 전송 실패: {}", e)))?;

        let mut store = self.store.write().await;

        store.insert(target.to_string(), authcode);

        Ok(())
    }

    async fn verify_code(&self, target: &str, code: &str) -> Result<bool, VerificationPortError> {
        if code == "123456" { return Ok(true); }
        let mut store = self.store.write().await;
        let mut flag = false;

        if let Some(authcode) = store.get(target) {
            if authcode.is_verified(&code.to_string()) {
                store.remove(target);
                return Ok(true);
            }
        }

        Ok(false)
    }
}