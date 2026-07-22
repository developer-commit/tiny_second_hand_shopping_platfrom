# ==========================================
# Stage 1: Frontend Builder (Leptos WASM 빌드)
# ==========================================
FROM rust:1.80-bookworm AS frontend-builder

# WASM 타겟 및 Trunk 설치 (프론트엔드 빌드 도구)
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk

WORKDIR /app
COPY . .

# 프론트엔드 폴더로 이동하여 WASM 릴리즈 빌드
WORKDIR /app/frontend
# Trunk가 index.html과 Rust 코드를 WASM으로 컴파일하여 /app/frontend/dist 에 생성합니다.
RUN trunk build --release

# ==========================================
# Stage 2: Backend Builder (Axum API 빌드)
# ==========================================
FROM rust:1.80-bookworm AS backend-builder

# 암호화(argon2, jsonwebtoken), 네트워크(reqwest) 빌드 필수 패키지
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# 백엔드 릴리즈 빌드
RUN cargo build --release -p backend

# ==========================================
# Stage 3: Runtime Stage (최종 실행 환경)
# ==========================================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# HTTPS 통신(reqwest) 및 암호화 처리에 필요한 런타임 라이브러리
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 1. 백엔드 바이너리 복사
COPY --from=backend-builder /app/target/release/backend /app/backend

# 2. 프론트엔드 정적 파일(WASM, HTML, CSS 등) 복사
# (Axum의 ServeDir이 이 폴더를 바라보도록 설정해야 합니다)
COPY --from=frontend-builder /app/frontend/dist /app/dist

# 업로드된 이미지(image 크레이트)가 저장될 로컬 볼륨 폴더 생성
RUN mkdir -p /app/uploads

# 환경 변수 설정
ENV RUST_LOG=info
ENV PORT=8080
# Axum 라우터에서 정적 파일을 서빙할 경로를 환경변수로 지정 (코드에서 사용 권장)
ENV STATIC_DIR=/app/dist

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s \
  CMD curl -f http://localhost:8080/health || exit 1

# 서버 실행
CMD ["/app/backend"]