# 중고 거래 플랫폼 시스템 설계서 (System Design Specification) - 보강본

## 0. 목표 설정 (Goal)
* **목표**: 사용자 간 안전하고 원활한 중고 물품 거래가 가능한 C2C 플랫폼 구축
* **핵심 가치**: 
  - 비트코인 캐시(BCH) 기반의 개인 지갑 및 안전결제(에스크로) 지원
  - 실시간 커뮤니케이션(전체 채팅 및 1:1 개인 채팅) 및 **실시간 알림** 제공
  - **리뷰 시스템과** 자동화된 모더레이션 시스템을 통한 **안전한 거래 생태계 조성**

---

## 1. 요구사항 도출 (Requirements)

1. **회원 관리**: 신규 가입, 로그인, **본인인증(2FA)**, 프로필 관리 및 마이페이지 기능
2. **상품 관리**: 상품 등록/수정/삭제, **카테고리 분류, 상품 상태(판매중/예약중/완료) 관리**
3. **신뢰 및 소통**: 플랫폼 공용/1:1 채팅, **거래 후 평점 및 리뷰 작성**
4. **알림 시스템**: **채팅, 결제 상태 변경, 신고 결과 등에 대한 실시간 푸시/웹 알림**
5. **신고/모더레이션**: 불량 유저/상품 신고 및 누적 신고 수 기반 자동 차단/휴면 전환
6. **지갑 및 결제**: BCH 지갑 충전/송금/출금, 에스크로 안전결제, 분쟁 중재 및 **자동 구매 확정 로직**
7. **검색 및 정렬**: 키워드 및 태그/카테고리 기반 검색, 정렬 조건 제공
8. **비기능 및 관리자**: 데이터 암호화(개인키 보안), 전역 API 권한 제어, 실시간 모니터링 패널

---

## 2. 시스템 설계 (System Design)

### 2.1. 기능 일반화 (System Specifications)

#### Module 1. 유저 및 계정 관리 (User & Account Management)
* **1.1. 회원가입 및 계정 생성**
  * 아이디 중복 확인 및 유일성 검증 기능
  * **스팸 방지를 위한 이메일 또는 SMS 본인 인증 기능 (추가)**
* **1.2. 인증 및 보안 (강화)**
  * 로그인 / 로그아웃 및 세션 관리
  * **지갑 출금 시 2단계 인증(2FA, OTP 등) 지원 (추가)**
* **1.3. 프로필 및 마이페이지**
  * 사용자 프로필, 소개글 작성 및 수정 기능
  * **유저 신뢰도 지수(평점) 노출 (추가)**

#### Module 2. 상품 관리 및 카탈로그 (Product Management System)
* **2.1. 상품 등록 및 상태 관리 (강화)**
  * 상품 정보(상품명, 가격, 사진, **카테고리**, 태그) DB 저장
  * **상품 상태 변경 기능 (판매중 ↔ 예약중 ↔ 판매완료) (추가)**
* **2.2. 상품 목록 및 상세 조회**
  * 목록 화면: 상품명, 가격, 썸네일, **상품 상태**, 조회수 노출
  * 상세 화면: 판매자 신뢰도, 전체 정보 노출 및 '채팅하기/안전결제' 분기 처리

#### Module 3. 검색, 정렬 및 알림 (Search & Notification System)
* **3.1. 상품 검색 기능**
  * 제목/태그 검색 및 **카테고리별 필터링 (추가)**
* **3.2. 알림 기능 (신규)**
  * **새로운 채팅 메시지 수신 시 웹/푸시 알림**
  * **에스크로 상태 변경(입금, 수령, 정산) 시 시스템 알림**

#### Module 4. 실시간 소통 및 리뷰 (Communication & Review System)
* **4.1. 채팅 시스템**
  * 공용 채널 채팅 및 1:1 개인 채팅 기능 (WebSocket 기반)
* **4.2. 평점 및 리뷰 (신규)**
  * **에스크로 거래 완료 시 상대방에 대한 리뷰 및 평점(1~5점) 작성 기능**
  * **작성된 리뷰에 따른 판매자/구매자 신뢰도 점수 자동 반영**

#### Module 5. 비트코인 캐시(BCH) 지갑 및 결제 (Wallet & Escrow System)
* **5.1. 지갑 및 송금 관리**
  * 유저별 BCH 지갑 생성 (보안을 위해 핫지갑/콜드지갑 분리 보관 권장)
  * **출금 시 네트워크 수수료(Network Fee) 계산 및 안내 로직 (추가)**
* **5.2. 안전결제(에스크로) 시스템 (강화)**
  * 플랫폼 중계 지갑으로 대금 예치 시 **상품 상태 자동 '예약중' 전환**
  * 구매자 수령 승인 시 대금 정산 및 플랫폼 수수료 차감
  * **자동 확정(Timeout): 구매자가 물품 수령 후 일정 기간(예: 3일) 미응답 시 자동 수령 승인 및 정산 (추가)**
  * 수령 거부 시 운영진 중재 및 환불 프로세스 진행

#### Module 6. 불량 유저 및 콘텐츠 필터링 (Moderation)
* **6.1. 자동 제재 및 상태 전환**
  * 특정 신고 횟수 초과 누적 시 상품 차단 / 유저 계정 휴면 전환
  * **악의적 신고 방지를 위해 1유저 1상품 1회 신고 제한 (추가)**

#### Module 7. 시스템 관리자 (Admin & System Operations)
* **7.1. 어드민 패널 및 모니터링**
  * 최고 관리자 API 전역 호출 권한
  * DB 실시간 현황, **누적 플랫폼 수수료 수익 조회 (추가)**
  * 분쟁 건에 대한 강제 정산/환불 트리거 기능

---

### 2.3. 웹페이지 설계 (Web Page Information Architecture)

| 페이지명 | 주요 기능 및 화면 구성 요소 | 연결 URL / 경로 |
| :--- | :--- | :--- |
| **메인 / 목록** | 카테고리/제목/태그 검색, 정렬 탭, 상품 카드(상품명, 가격, 썸네일, **상품 상태, 조회수** 노출), 공용채팅 버튼 | `/` |
| **회원가입** | 기본 정보 입력, **아이디 중복 확인 및 유일성 검증**, 이메일/SMS 본인 인증 | `/signup` |
| **로그인 / 로그아웃** | 아이디/비밀번호 인증 및 세션 관리 (로그아웃은 공통 헤더/메뉴에 포함) | `/login`, `/logout` |
| **마이페이지** | 내 프로필, 나의 평점/리뷰(신뢰도 지수), **계정 보안(비밀번호, 2FA 설정)**, 상품/거래 내역 관리 | `/mypage` |
| **지갑 대시보드** | **BCH 잔액 조회, 입금용 지갑 주소 발급/확인**, 전체 입출금 트랜잭션 내역 | `/wallet` |
| **지갑 출금** | 외부 지갑 주소 입력, **네트워크 수수료(Network Fee) 계산 및 안내**, 출금 전 2FA(OTP) 인증 모달 | `/wallet/withdraw` |
| **상품 등록** | 상품명, 카테고리, 가격, 이미지, 태그 입력 DB 저장 폼 | `/products/new` |
| **상품 수정** | 기존 등록 상품 정보 수정 폼, **상품 상태(판매중/예약중/판매완료) 변경 버튼** | `/products/:id/edit` |
| **상품 상세** | 상세 정보, 판매자 신뢰도, 1:1 채팅/안전결제 버튼, 상품 상태 뱃지, 신고하기 폼(**1유저 1상품 1회 제한**) | `/products/:id` |
| **실시간 채팅** | 공용 채팅 탭, 1:1 채팅 목록, 거래 연동(채팅방 내 송금/결제 버튼) | `/chat` |
| **안전결제** | 대금 예치 상태, 수령 승인, 수령 거부 폼, **자동 확정(Timeout) 기한 안내**, 거래 완료 후 **상대방 리뷰/평점 작성 폼** | `/escrow/:trade_id` |
| **알림 내역** | 시스템 알림(에스크로 상태 변경), 채팅 메시지 도착 알림 내역 모아보기 | `/notifications` |
| **관리자 어드민** | DB 조회, 자동 제재 현황, **분쟁 중재(강제 정산/환불 트리거) 패널**, 플랫폼 지갑 모니터링, **수수료 수익 통계** | `/admin` |


### 2.4 데이터베이스 스키마 
```sql
-- ==========================================
-- Module 1. 유저 및 계정 관리 (Users & Accounts)
-- ==========================================
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,    -- 아이디 (중복 확인 및 유일성)
    password_hash VARCHAR(255) NOT NULL,
    email VARCHAR(100) UNIQUE,               -- 이메일 인증용
    phone VARCHAR(20) UNIQUE,                -- SMS 인증용
    is_verified BOOLEAN DEFAULT FALSE,       -- 본인 인증 여부
    bio TEXT,                                -- 소개글
    trust_score DECIMAL(3, 2) DEFAULT 0.00,  -- 유저 신뢰도 지수 (평점)
    is_2fa_enabled BOOLEAN DEFAULT FALSE,    -- 2단계 인증(OTP) 활성화 여부
    two_factor_secret VARCHAR(255),          -- 2FA 시크릿 키
    role VARCHAR(20) DEFAULT 'user',         -- 권한 (user, admin)
    status VARCHAR(20) DEFAULT 'active',     -- 계정 상태 (active, dormant, suspended)
    reported_count INT DEFAULT 0,            -- 누적 신고당한 횟수
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ==========================================
-- Module 5. 지갑 관리 (Wallets & Transactions)
-- ==========================================
CREATE TABLE wallets (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT UNIQUE NOT NULL REFERENCES users(id),
    bch_address VARCHAR(100) UNIQUE NOT NULL, -- 입금용 BCH 지갑 주소
    balance DECIMAL(18, 8) DEFAULT 0.00000000, -- BCH 잔액
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE wallet_transactions (
    id BIGSERIAL PRIMARY KEY,
    wallet_id BIGINT NOT NULL REFERENCES wallets(id),
    tx_type VARCHAR(20) NOT NULL,            -- 타입 (deposit, withdrawal)
    amount DECIMAL(18, 8) NOT NULL,          -- 송금액
    network_fee DECIMAL(18, 8) DEFAULT 0,    -- 출금 시 네트워크 수수료
    tx_hash VARCHAR(255) UNIQUE,             -- 블록체인 트랜잭션 해시
    status VARCHAR(20) DEFAULT 'pending',    -- 상태 (pending, confirmed, failed)
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ==========================================
-- Module 2 & 3. 상품 관리 및 카탈로그 (Products)
-- ==========================================
CREATE TABLE products (
    id BIGSERIAL PRIMARY KEY,
    seller_id BIGINT NOT NULL REFERENCES users(id),
    title VARCHAR(255) NOT NULL,             -- 상품명
    description TEXT NOT NULL,               -- 상품 상세 설명
    price DECIMAL(18, 8) NOT NULL,           -- 가격 (BCH)
    category VARCHAR(50) NOT NULL,           -- 카테고리
    status VARCHAR(20) DEFAULT 'on_sale',    -- 상태 (on_sale, reserved, sold)
    view_count INT DEFAULT 0,                -- 조회수
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE product_images (
    id BIGSERIAL PRIMARY KEY,
    product_id BIGINT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    image_url VARCHAR(255) NOT NULL,         -- S3 등 이미지 저장 경로
    is_thumbnail BOOLEAN DEFAULT FALSE,      -- 썸네일 여부
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE product_tags (
    id BIGSERIAL PRIMARY KEY,
    product_id BIGINT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    tag_name VARCHAR(50) NOT NULL            -- 태그 검색용
);
CREATE INDEX idx_product_tags_name ON product_tags(tag_name);

-- ==========================================
-- Module 5. 안전결제 / 에스크로 (Escrow Trades)
-- ==========================================
CREATE TABLE escrow_trades (
    id BIGSERIAL PRIMARY KEY,
    product_id BIGINT NOT NULL REFERENCES products(id),
    buyer_id BIGINT NOT NULL REFERENCES users(id),
    seller_id BIGINT NOT NULL REFERENCES users(id),
    amount DECIMAL(18, 8) NOT NULL,          -- 예치된 대금
    platform_fee DECIMAL(18, 8) NOT NULL,    -- 플랫폼 수수료
    status VARCHAR(20) DEFAULT 'deposited',  -- 상태 (deposited, received, disputed, settled, refunded)
    auto_confirm_at TIMESTAMP WITH TIME ZONE,-- 자동 확정(Timeout) 기준 일시 (예: 예치 후 +3일)
    dispute_reason TEXT,                     -- 수령 거부(분쟁) 사유
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ==========================================
-- Module 4. 평점 및 리뷰 (Reviews)
-- ==========================================
CREATE TABLE reviews (
    id BIGSERIAL PRIMARY KEY,
    trade_id BIGINT UNIQUE NOT NULL REFERENCES escrow_trades(id), -- 1거래 1리뷰
    reviewer_id BIGINT NOT NULL REFERENCES users(id),
    reviewee_id BIGINT NOT NULL REFERENCES users(id),
    rating INT CHECK (rating >= 1 AND rating <= 5), -- 1~5점
    comment TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ==========================================
-- Module 4. 채팅 시스템 (Chat System)
-- ==========================================
CREATE TABLE chat_rooms (
    id BIGSERIAL PRIMARY KEY,
    product_id BIGINT REFERENCES products(id) ON DELETE SET NULL, -- 1:1 채팅 시 연동된 상품
    room_type VARCHAR(20) DEFAULT 'private', -- (public, private)
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE chat_participants (
    room_id BIGINT NOT NULL REFERENCES chat_rooms(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (room_id, user_id)
);

CREATE TABLE chat_messages (
    id BIGSERIAL PRIMARY KEY,
    room_id BIGINT NOT NULL REFERENCES chat_rooms(id) ON DELETE CASCADE,
    sender_id BIGINT NOT NULL REFERENCES users(id),
    message TEXT NOT NULL,
    is_read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ==========================================
-- Module 6. 불량 유저 신고 (Reports)
-- ==========================================
CREATE TABLE reports (
    id BIGSERIAL PRIMARY KEY,
    reporter_id BIGINT NOT NULL REFERENCES users(id),
    product_id BIGINT NOT NULL REFERENCES products(id),
    reason TEXT NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',    -- (pending, reviewed, action_taken)
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    -- [핵심 로직] 악의적 신고 방지: 1유저 1상품 1회 신고 제한
    UNIQUE (reporter_id, product_id)
);

-- ==========================================
-- Module 3. 알림 시스템 (Notifications)
-- ==========================================
CREATE TABLE notifications (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type VARCHAR(30) NOT NULL,               -- (chat, escrow_update, system, warning)
    reference_id BIGINT,                     -- 관련된 엔티티의 ID (trade_id, message_id 등)
    message TEXT NOT NULL,
    is_read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ==========================================
-- Module 7. 관리자 및 플랫폼 통계 (Platform Stats)
-- ==========================================
CREATE TABLE platform_stats (
    id SERIAL PRIMARY KEY,
    total_fee_collected DECIMAL(18, 8) DEFAULT 0.00000000, -- 누적 플랫폼 수수료 수익
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

### 2.5 open api 문서 작성
open.yaml 문서 참고

### 2.6 구현되어야 하는 보안 요소
1. 계정 및 인증/인가 보안 (Authentication & Authorization)
  - 비밀번호 단방향 암호화 저장 (password_hash: Argon2, Bcrypt 등 안전한 해시 알고리즘 적용)
  - 신규 가입 시 스팸 및 위장 가입 방지를 위한 이메일/SMS 본인 인증
  - 세션 및 인증 토큰(JWT 등) 만료 및 무결성 관리
  - 2단계 인증 (2FA / OTP) 지원 및 지갑 출금/보안 설정 시 필수 검증
  - 2FA 시크릿 키(two_factor_secret) 양방향 암호화 보관
  - 역할 기반 접근 제어 (RBAC: 'user', 'admin' 권한 분리 및 엔드포인트 검증)
  - 계정 상태(status: active, dormant, suspended)에 따른 API 접근 제한

2. 가상자산 및 지갑 보안 (Wallet & Crypto Security)
  - 핫지갑(Hot Wallet)과 콜드지갑(Cold Wallet)의 분리 보관 및 운영
  - 개인키(Private Key) 및 지갑 관련 민감 데이터의 KMS/HSM 기반 암호화 보관
  - BCH 출금 요청 시 2FA(OTP) 추가 인증 필수화
  - 블록체인 트랜잭션 해시(tx_hash) 검증을 통한 무결성 확보 및 이중 지불 방지

3. 거래 및 에스크로 보안 (Transaction & Escrow Security)
  - 제3자 에스크로(안전결제) 예치 시스템을 통한 구매자/판매자 대금 이동 보호
  - 자동 구매 확정(Timeout) 및 상태 변경 로직의 변조 방지 처리
  - 분쟁 발생 시 운영진 강제 정산/환불 기능에 대한 접근 권한 제한 및 감사 로그 기록

4. 모더레이션 및 어뷰징 방지 (Anti-Abuse & Moderation)
  - 신고 누적에 따른 불량 유저 계정 휴면 전환 및 상품 자동 차단 로직
  - 악의적/중복 신고 방지 (1유저 1상품 1회 신고 DB UNIQUE 제약조건 적용)
  - 유저 신뢰도 지수(trust_score) 자동 계산을 통한 신뢰 생태계 유지

5. 시스템 및 API 보안 (System & API Security)
  - 관리자(Admin) 전용 API 전역 접근 제어 (Global Authorization Middleware)
  - 개인정보 및 민감 데이터 DB 저장 시 데이터 암호화 (Data-at-Rest Protection)
  - API 요청에 대한 Rate Limiting 및 스팸/DDoS 방지

# 3. 기술 스택 결정

> **기술 스택 선정 배경**  
> 프로젝트를 All Vibe로 진행함에 따라, 코드 검증 단계에서 컴파일러의 강한 타입 체크와 안전성 보장을 활용하고자 프론트엔드(Frontend)와 백엔드(Backend) 모두 **Rust**를 전면 도입하기로 결정하였습니다.

---

프로젝트의 `Cargo.toml`에 직접 등록하여 활용하는 분야별 최적화 라이브러리 목록입니다.

### 1. 프론트엔드 (Frontend - Leptos)
| 라이브러리 | 설명 | 저장소 링크 |
| :--- | :--- | :--- |
| `leptos` | Leptos 코어 (반응형 상태 관리 및 UI 컴포넌트) | [GitHub](https://github.com/leptos-rs/leptos) |
| `leptos_router` | 클라이언트 사이드 SPA 라우터 | [GitHub](https://github.com/leptos-rs/leptos) |
| `leptos_meta` | HTML Head, Title, Meta 태그 관리 | [GitHub](https://github.com/leptos-rs/leptos) |

### 2. 백엔드 코어 & 웹 프레임워크 (Backend Core)
| 라이브러리 | 설명 | 저장소 링크 |
| :--- | :--- | :--- |
| `tokio` | 비동기 런타임 | [GitHub](https://github.com/tokio-rs/tokio) |
| `axum` | 웹 프레임워크 및 WebSocket 지원 | [GitHub](https://github.com/tokio-rs/axum) |
| `tower-http` | CORS, Rate Limiting, 로컬 정적파일 서빙(ServeDir) | [GitHub](https://github.com/tower-rs/tower-http) |

### 3. 데이터베이스 & ORM (Database)
| 라이브러리 | 설명 | 저장소 링크 |
| :--- | :--- | :--- |
| `sea-orm` | 비동기 PostgreSQL ORM | [GitHub](https://github.com/SeaQL/sea-orm) |

### 4. 인증, 권한 및 보안 (Auth & Security)
| 라이브러리 | 설명 | 저장소 링크 |
| :--- | :--- | :--- |
| `argon2` | 비밀번호 단방향 해싱 | [GitHub](https://github.com/RustCrypto/password-hashes) |
| `jsonwebtoken` | JWT 세션/인증 토큰 | [GitHub](https://github.com/Keats/jsonwebtoken) |
| `totp-rs` | 2FA / OTP 검증 | [GitHub](https://github.com/sMouS/totp-rs) |
| `aes-gcm` | 2FA 시크릿 키 & 지갑 개인키 DB 암호화 | [GitHub](https://github.com/RustCrypto/AEADs) |
| `validator` | 이메일/폼 데이터 유효성 검증 | [GitHub](https://github.com/Keats/validator) |

### 5. 가상자산 지갑 및 외부 통신 (BCH Light Client)
| 라이브러리 | 설명 | 저장소 링크 |
| :--- | :--- | :--- |
| `bip39` | HD 지갑 니모닉(시드 문구) 생성 및 복구 | [GitHub](https://github.com/kakeing/bip39-rs) |
| `rust-bitcoin` | 오프라인 트랜잭션 생성 및 키 파싱 | [GitHub](https://github.com/rust-bitcoin/rust-bitcoin) |
| `cashaddr` | BCH 전용 CashAddr 주소 변환 (`bitcoincash:q...`) | [GitHub](https://github.com/lokamanta/cashaddr) |
| `reqwest` | 퍼블릭 API / Electrum HTTP 게이트웨이 통신 | [GitHub](https://github.com/seanmonstar/reqwest) |

### 6. 실시간 메시징 & 백그라운드 스케줄링 (Real-time & Worker)
| 라이브러리 | 설명 | 저장소 링크 |
| :--- | :--- | :--- |
| `redis` | 채팅/알림 Pub/Sub 메시징 (로컬 Redis) | [GitHub](https://github.com/redis-rs/redis-rs) |
| `tokio-cron-scheduler` | 3일 타임아웃 자동 확정 크론 작업 | [GitHub](https://github.com/msrd0/tokio-cron-scheduler) |

### 7. 로컬 미디어 파일 처리 (Local Storage)
| 라이브러리 | 설명 | 저장소 링크 |
| :--- | :--- | :--- |
| `image` | 업로드 이미지 썸네일 리사이징 및 로컬 저장 | [GitHub](https://github.com/image-rs/image) |

### 8. 공통 유틸리티 (Utilities)
| 라이브러리 | 설명 | 저장소 링크 |
| :--- | :--- | :--- |
| `serde` | 직렬화 / 역직렬화 (프론트/백엔드 공통) | [GitHub](https://github.com/serde-rs/serde) |
| `serde_json` | JSON 파싱 | [GitHub](https://github.com/serde-rs/json) |
| `chrono` | 날짜 및 타임스탬프 처리 | [GitHub](https://github.com/chronotope/chrono) |
| `dotenvy` | `.env` 환경 변수 로드 | [GitHub](https://github.com/allan2/dotenvy) |
| `tracing` | 로깅 및 모니터링 | [GitHub](https://github.com/tokio-rs/tracing) |
| `thiserror` | API 에러 커스텀 정의 | [GitHub](https://github.com/dtolnay/thiserror) |

# 4. 모델 구조 데모
```
├── Cargo.lock
├── Cargo.toml
├── crates
│   ├── backend
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── db
│   │       │   ├── chat_message.rs
│   │       │   ├── escrow_trade.rs
│   │       │   ├── mod.rs
│   │       │   ├── platform_stats.rs
│   │       │   ├── product.rs
│   │       │   └── wallet_transaction.rs
│   │       ├── main.rs
│   │       └── utils
│   │           ├── mod.rs
│   │           └── security.rs
│   ├── frontend
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── main.rs
│   │       └── models
│   │           ├── escrow_model.rs
│   │           ├── mod.rs
│   │           ├── notification_store.rs
│   │           └── product_model.rs
│   └── shared
│       ├── Cargo.toml
│       └── src
│           ├── dto
│           │   ├── admin_dto.rs
│           │   ├── chat_dto.rs
│           │   ├── escrow_dto.rs
│           │   ├── mod.rs
│           │   ├── noti_dto.rs
│           │   ├── product_dto.rs
│           │   ├── report_dto.rs
│           │   └── transaction_dto.rs
│           ├── dto.rs
│           └── lib.rs
├── openapi.yaml
├── plan.md
```
3대 보안 원칙(식별자 난독화, 스키마 은닉, 관심사 분리)을 적용하여 모델 구조, dto 구조를 결정한다.

# 5. 프로젝트 구조 구체화
```
# Role
당신은 최고 수준의 Rust 풀스택 아키텍처 설계자이자 보안 전문가입니다. 
당신의 목표는 주어진 프로젝트 구조와 라이브러리 스택, 핵심 보안 규격을 바탕으로 비트코인 캐시(BCH) 기반 중고 거래 플랫폼의 **완벽한 시스템 뼈대(Scaffold)와 아키텍처 구조화**를 수행하는 것입니다.
openapi.yaml, 프로젝트의 구조의 논리적 모순을 발견시, 수정사항을 plan.md에 추가후 수정해도 무방합니다.

*주의: 실제 세부 비즈니스 로직(내부 작동 코드)을 구현하는 것이 아닙니다.* 데이터 구조(Struct), API 인터페이스(Trait/Handler), 프론트엔드 라우팅 및 상태 래퍼의 시그니처를 설계하여 **견고한 구조(Blueprint)를 만드는 것**이 핵심입니다.

# Context
- 당신은 현재 프로젝트의 디렉토리 구조(Workspace: frontend, backend, shared)를 읽고 파악할 수 있습니다.
- 백엔드는 `Axum`과 `SeaORM`, 프론트엔드는 `Leptos`, 통신 규격은 `shared` 크레이트의 DTO로 철저히 분리되어 있습니다.
- plan.md에는 프로젝트의 목표가 담겨 있습니다.
- openapi yaml에는 api 계약이 적혀있습니다.

---

# 🛡️ 구조적 보안 및 아키텍처 원칙 (뼈대에 반드시 반영)
코드를 구조화할 때 아래 원칙이 **타입 시스템과 모듈 구조**에 명확히 드러나야 합니다.

1. **식별자 난독화 (Opaque IDs)**
   - `shared` DTO 구조체에는 순차 ID(`i64`)가 존재해선 안 됩니다. 반드시 `String` (또는 `OpaqueId` 같은 커스텀 타입)으로 정의하세요.
   - `backend/src/utils/security.rs`에 난독화 유틸리티의 함수 시그니처(`pub fn obfuscate`, `pub fn deobfuscate`)와 커스텀 에러(`thiserror`) 구조만 정의하세요.

2. **스키마 은닉 (Schema Hiding) 및 관심사 분리**
   - **Shared**: `validator` 매크로가 적용된 DTO와 직렬화(`serde`) 구조체만 정의. (DB 컬럼명과 다르게 네이밍)
   - **Backend (Entity)**: `sea-orm` 모델 구조체와 `TryFrom<Model> for DTO` 변환 트레잇의 시그니처 작성.
   - **Frontend (Model)**: `shared` DTO를 받아 `leptos`의 `RwSignal`이나 `Resource`로 감싸는 프론트엔드용 상태 구조체(Wrapper) 설계.

3. **인증, 권한, 지갑 보안의 타입화**
   - 비밀번호, 2FA 시크릿 키, 지갑 개인키는 일반 `String`이 아닌 보안 래퍼 타입(예: `SecretString`, `EncryptedKey`)으로 정의하여 메모리 유출을 방지하는 구조를 보여주세요.
   - 글로벌 미들웨어(Auth/RBAC)가 적용될 Axum 라우터의 뼈대(Router builder)를 설계하세요.

4. **디자인 패턴 적용 (외부 의존성 격리 및 비즈니스 로직 캡슐화)**
   - 비즈니스 로직(Service)과 인프라스트럭처(DB, 네트워크)를 철저히 분리하세요.
   - 특히 지갑(Wallet), 외부 BCH 네트워크 통신, 결제 모듈 등은 외부 환경 변화에 대비하여 **어댑터 패턴(Ports & Adapters / Hexagonal)**을 적용해야 합니다. Rust의 `trait`(Port)으로 인터페이스를 먼저 정의하고, 이를 구현하는 `struct`(Adapter)를 인프라 계층에 배치하세요.

---

# 🚀 작업 지시 (Scaffolding Execution)

내부 로직은 `todo!("구체적인 로직 설명")` 또는 `unimplemented!()`로 비워두고, **인터페이스와 구조**에 집중하여 코드를 생성합니다. 사용자가 지정하는 **[Target Domain]**에 대해 아래 5계층의 뼈대를 작성하세요.

### [출력해야 할 구조화 영역]
1. **[Shared] API Contract (DTOs & Enums)**: 
   - 프론트와 백엔드가 통신할 Request/Response 데이터 구조. 유효성 검증(`validator`) 어노테이션 포함.
2. **[Backend] Controller Layer (Axum Handlers & Router)**: 
   - API 엔드포인트 함수 시그니처, 의존성 주입(State), 사용될 미들웨어(Auth) 구조. (HTTP I/O만 담당)
3. **[Backend] Service & Domain Layer (Business Logic & Ports)**: 
   - 비즈니스 핵심 로직 시그니처. 
   - 외부 인프라(DB, 지갑 통신)를 추상화한 의존성 주입용 `trait` (Port) 정의.
4. **[Backend] Infrastructure Layer (SeaORM Model & Adapters)**: 
   - DB 스키마 구조체 및 DTO 변환 시그니처 (`TryFrom`/`Into`).
   - Service 계층에서 정의한 `trait`을 구현하여 실제 외부 API(지갑/BCH 네트워크)와 통신하는 어댑터(Adapter) 구조체.
5. **[Frontend] View & State Layer (Leptos Models & Router)**: 
   - 상태 관리를 위한 래퍼 구조체(Signal/Resource)와 페이지 컴포넌트(`#[component]`) 뼈대.

### 📝 코드 출력 규칙
- 각 파일 및 코드 블록 상단에 주석으로 **"이 구조가 어떤 아키텍처/보안 목적을 달성하는지"** 간략히 설명하세요.
- 세부 비즈니스 로직 작성에 토큰을 낭비하지 마세요. 대신 **데이터의 흐름(Flow)과 타입 시스템(Type System)**이 어떻게 안전성을 보장하는지 명확히 보여주는 데 집중하세요.
- 프로젝트 디렉토리 구조에 맞는 정확한 파일 경로를 주석에 명시한 뒤 코드 블록(```rust)을 작성하세요.
```
프로젝트 구조 결과물
```
├── backend
│   ├── Cargo.toml
│   └── src
│       ├── db
│       │   ├── chat_message.rs
│       │   ├── entity
│       │   │   ├── chat_message.rs
│       │   │   ├── chat_room.rs
│       │   │   ├── escrow_trade.rs
│       │   │   ├── mod.rs
│       │   │   ├── notification.rs
│       │   │   ├── platform_stats.rs
│       │   │   ├── product_image.rs
│       │   │   ├── product_tag.rs
│       │   │   ├── product.rs
│       │   │   ├── report.rs
│       │   │   ├── review.rs
│       │   │   ├── user.rs
│       │   │   ├── wallet_transaction.rs
│       │   │   └── wallet.rs
│       │   ├── escrow_trade.rs
│       │   ├── mod.rs
│       │   ├── platform_stats.rs
│       │   ├── product.rs
│       │   └── wallet_transaction.rs
│       ├── handlers
│       │   ├── admin_handler.rs
│       │   ├── auth_handler.rs
│       │   ├── chat_handler.rs
│       │   ├── escrow_handler.rs
│       │   ├── mod.rs
│       │   ├── notification_handler.rs
│       │   ├── product_handler.rs
│       │   ├── report_handler.rs
│       │   ├── review_handler.rs
│       │   ├── user_handler.rs
│       │   └── wallet_handler.rs
│       ├── infra
│       │   ├── bch_network_adapter.rs
│       │   ├── bch_wallet_adapter.rs
│       │   ├── mod.rs
│       │   └── redis_pubsub_adapter.rs
│       ├── main.rs
│       ├── middleware
│       │   ├── auth.rs
│       │   ├── mod.rs
│       │   └── rbac.rs
│       ├── ports
│       │   ├── bch_network_port.rs
│       │   ├── mod.rs
│       │   ├── notification_port.rs
│       │   ├── pubsub_port.rs
│       │   └── wallet_port.rs
│       ├── router.rs
│       ├── service
│       │   ├── admin_service.rs
│       │   ├── auth_service.rs
│       │   ├── chat_service.rs
│       │   ├── escrow_service.rs
│       │   ├── mod.rs
│       │   ├── notification_service.rs
│       │   ├── product_service.rs
│       │   ├── report_service.rs
│       │   ├── review_service.rs
│       │   ├── user_service.rs
│       │   └── wallet_service.rs
│       ├── state.rs
│       └── utils
│           ├── auth.rs
│           ├── mod.rs
│           └── security.rs
├── frontend
│   ├── Cargo.toml
│   └── src
│       ├── main.rs
│       ├── models
│       │   ├── auth_model.rs
│       │   ├── escrow_model.rs
│       │   ├── mod.rs
│       │   ├── notification_store.rs
│       │   ├── product_model.rs
│       │   └── wallet_model.rs
│       ├── pages
│       │   ├── admin.rs
│       │   ├── chat.rs
│       │   ├── escrow.rs
│       │   ├── home.rs
│       │   ├── login.rs
│       │   ├── mod.rs
│       │   ├── mypage.rs
│       │   ├── notifications.rs
│       │   ├── product_detail.rs
│       │   ├── product_edit.rs
│       │   ├── product_new.rs
│       │   ├── signup.rs
│       │   └── wallet.rs
│       └── router.rs
└── shared
    ├── Cargo.toml
    └── src
        ├── dto
        │   ├── admin_dto.rs
        │   ├── chat_dto.rs
        │   ├── escrow_dto.rs
        │   ├── mod.rs
        │   ├── noti_dto.rs
        │   ├── product_dto.rs
        │   ├── report_dto.rs
        │   ├── review_dto.rs
        │   ├── transaction_dto.rs
        │   └── user_dto.rs
        ├── lib.rs
        └── types.rs
```


# 4. 시스템 구현
backend 에 대한 구현 지시 프롬프트
```
# Role
당신은 엄격한 규율과 아키텍처 패턴을 준수하며, 파일 시스템과 도구를 자유롭게 넘나드는(Anti-gravity) 시니어 Rust 백엔드 에이전트입니다. 
현재 당신의 목표는 `backend` 크레이트 내부의 `todo!()` 및 `unimplemented!()` 매크로를 실제 로직으로 완성하는 것입니다.

# Architecture & Context Awareness
이 프로젝트는 Workspace 구조로 되어 있으며, 백엔드 로직을 구현할 때 다음 아키텍처 흐름을 반드시 숙지하십시오.
- **Shared DTO:** 요청/응답 데이터 구조는 `shared/src/dto/`에 정의되어 있습니다. 백엔드에서 자체 DTO를 만들지 말고 `shared::dto::*`를 Import하여 사용하세요.
- **Layered Flow:** HTTP 요청 흐름은 `handlers` -> `service` -> `ports`(또는 `db/infra`) 순으로 이어집니다.
- **State:** 의존성 주입은 `backend/src/state.rs`를 통해 이루어집니다.
- **Rule** 

# workflow (자율 작업 프로세스)
1. **Search (탐색):** `backend/src` 내에서 `todo!()`나 `unimplemented!()`가 있는 파일을 찾습니다.
2. **Context Tracking (동적 컨텍스트 추적):** 
   - 대상 파일이 속한 계층에 맞춰 연결된 `service`, `db/entity`, `ports`, `shared/src/dto/` 파일들을 **스스로 찾아서 읽으십시오(Read).**
   - 인증/권한 로직이 필요하다면 `utils/auth.rs` 및 `middleware/`를 참조하십시오.
3. **Write (직접 파일 수정):** 코드 전체를 채팅창에 출력하여 설명을 늘어놓지 마십시오. 도구(Tool)를 사용하여 **대상 파일의 코드만 직접 수정(Edit/Write)** 하십시오.
4. **Verify (자율 검증):** 파일을 수정한 후에는 반드시 터미널 명령어를 통해 `cargo check -p backend`를 실행하고, 에러가 발생하면 로그를 읽고 스스로 수정하십시오.

# Strict Rules (절대 규칙)
1. **시그니처 및 인라인 명세(Spec) 절대 준수**: 
   - 기존 함수의 이름, 매개변수 타입, 반환 타입은 절대 변경하지 마십시오.
   - **[가장 중요]** `todo!("...")` 내부에 작성된 지시사항(예: 반환할 HTTP 상태 코드, 호출할 서비스 메서드 등)은 **개발자의 절대적인 명세**입니다. 이를 최우선으로 따르며, 지시되지 않은 불필요한 에러 핸들링이나 과도한 로직을 임의로 추가하지 마십시오.
2. **구조 변경 금지**: 새로운 모듈을 만들거나 파일 분리를 제안하지 마십시오. 오직 주어진 파일의 스코프 내에서만 작업하십시오.
3. **모듈 트리 훼손 금지**: `use` 구문을 추가할 때, 기존 프로젝트 구조(`shared` 크레이트 포함)에 맞게 정확한 경로를 명시하십시오.

# Task Initialization
이제 `backend/src` 폴더 내에서 첫 번째로 구현할 대상을 찾아 탐색을 시작하고, 위 Workflow에 따라 구현 후 `cargo check` 결과까지 확인하여 완료 상태를 보고하십시오.
```

영문 프롬프트
```
# Role
You are a Senior Rust Backend Agent who adheres strictly to discipline and architectural patterns, seamlessly traversing file systems and tools (Anti-gravity).  
Your current goal is to complete all `todo!()` and `unimplemented!()` macros inside the `backend` crate with actual working logic.

# Architecture & Context Awareness
This project uses a Workspace structure. When implementing backend logic, you must strictly understand and follow the architectural flow below:
- **Shared DTO:** Request/response data structures are defined in `shared/src/dto/`. Do not create custom DTOs in the backend; instead, import and use `shared::dto::*`.
- **Layered Flow:** The HTTP request flow follows this sequence: `handlers` -> `service` -> `ports` (or `db/infra`).
- **State:** Dependency injection is handled through `backend/src/state.rs`.

# Workflow
1. **Search:** Locate files containing `todo!()` or `unimplemented!()` macros within `backend/src`.
2. **Context Tracking:** 
   - Based on the layer of the target file, autonomously locate and read connected files in `service`, `db/entity`, `ports`, and `shared/src/dto/`.
   - If authentication/authorization logic is required, refer to `utils/auth.rs` and `middleware/`.
3. **Write:** Do not dump full code blocks or lengthy explanations into the chat window. Use tools to directly modify (edit/write) only the target file.
4. **Verify:** After modifying a file, always run `cargo check -p backend` via terminal command. If compilation errors occur, read the logs and fix them autonomously.

# Strict Rules
1. **Strictly Preserve Signatures and Inline Specifications:**
   - Never alter existing function names, parameter types, or return types.
   - **[MOST IMPORTANT]** Instructions written inside `todo!("...")` (e.g., HTTP status codes to return, service methods to call, etc.) serve as absolute developer specifications. Follow them with top priority, and do not arbitrarily add unrequested error handling or excessive logic.
2. **Do Not Alter Structure:** Do not create new modules or suggest splitting files. Work strictly within the scope of the given files.
3. **Preserve Module Hierarchy:** When adding `use` statements, specify exact paths consistent with the existing project structure (including the `shared` crate).

# Task Initialization
Now, begin searching within `backend/src` for the first implementation target. Proceed according to the Workflow above, implement the logic, verify with `cargo check`, and report the completion status.
```