use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let sql = r#"
-- ==========================================
-- Module 1. 유저 및 계정 관리 (Users & Accounts)
-- ==========================================
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    email VARCHAR(100) UNIQUE,
    phone VARCHAR(20) UNIQUE,
    is_verified BOOLEAN DEFAULT FALSE,
    bio TEXT,
    trust_score DECIMAL(3, 2) DEFAULT 0.00,
    is_2fa_enabled BOOLEAN DEFAULT FALSE,
    two_factor_secret VARCHAR(255),
    role VARCHAR(20) DEFAULT 'user',
    status VARCHAR(20) DEFAULT 'active',
    reported_count INT DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ==========================================
-- Module 5. 지갑 관리 (Wallets & Transactions)
-- ==========================================
CREATE TABLE wallets (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT UNIQUE NOT NULL REFERENCES users(id),
    eth_address VARCHAR(100) UNIQUE NOT NULL,
    balance NUMERIC(78, 0) DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE wallet_transactions (
    id BIGSERIAL PRIMARY KEY,
    wallet_id BIGINT NOT NULL REFERENCES wallets(id),
    tx_type VARCHAR(20) NOT NULL,
    amount NUMERIC(78, 0) NOT NULL,
    network_fee NUMERIC(78, 0) DEFAULT 0,
    tx_hash VARCHAR(255) UNIQUE,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ==========================================
-- Module 2 & 3. 상품 관리 및 카탈로그 (Products)
-- ==========================================
CREATE TABLE products (
    id BIGSERIAL PRIMARY KEY,
    seller_id BIGINT NOT NULL REFERENCES users(id),
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    price NUMERIC(78, 0) NOT NULL,
    category VARCHAR(50) NOT NULL,
    status VARCHAR(20) DEFAULT 'on_sale',
    view_count INT DEFAULT 0,
    currency VARCHAR(10) NOT NULL DEFAULT 'ETH',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE product_images (
    id BIGSERIAL PRIMARY KEY,
    product_id BIGINT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    image_url VARCHAR(255) NOT NULL,
    is_thumbnail BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE product_tags (
    id BIGSERIAL PRIMARY KEY,
    product_id BIGINT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    tag_name VARCHAR(50) NOT NULL
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
    amount NUMERIC(78, 0) NOT NULL,
    platform_fee NUMERIC(78, 0) NOT NULL,
    status VARCHAR(20) DEFAULT 'deposited',
    auto_confirm_at TIMESTAMP WITH TIME ZONE,
    dispute_reason TEXT,
    currency VARCHAR(10) NOT NULL DEFAULT 'ETH',
    blockchain_tx_hash VARCHAR(66),
    contract_trade_id BIGINT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ==========================================
-- Module 4. 평점 및 리뷰 (Reviews)
-- ==========================================
CREATE TABLE reviews (
    id BIGSERIAL PRIMARY KEY,
    trade_id BIGINT UNIQUE NOT NULL REFERENCES escrow_trades(id),
    reviewer_id BIGINT NOT NULL REFERENCES users(id),
    reviewee_id BIGINT NOT NULL REFERENCES users(id),
    rating INT CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ==========================================
-- Module 4. 채팅 시스템 (Chat System)
-- ==========================================
CREATE TABLE chat_rooms (
    id BIGSERIAL PRIMARY KEY,
    product_id BIGINT REFERENCES products(id) ON DELETE SET NULL,
    room_type VARCHAR(20) DEFAULT 'private',
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
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (reporter_id, product_id)
);

-- ==========================================
-- Module 3. 알림 시스템 (Notifications)
-- ==========================================
CREATE TABLE notifications (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type VARCHAR(30) NOT NULL,
    reference_id BIGINT,
    message TEXT NOT NULL,
    is_read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ==========================================
-- Module 7. 관리자 및 플랫폼 통계 (Platform Stats)
-- ==========================================
CREATE TABLE platform_stats (
    id SERIAL PRIMARY KEY,
    total_fee_collected DECIMAL(18, 8) DEFAULT 0.00000000,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
        "#;
        manager.get_connection().execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let sql = r#"
            DROP TABLE IF EXISTS platform_stats CASCADE;
            DROP TABLE IF EXISTS notifications CASCADE;
            DROP TABLE IF EXISTS reports CASCADE;
            DROP TABLE IF EXISTS chat_messages CASCADE;
            DROP TABLE IF EXISTS chat_participants CASCADE;
            DROP TABLE IF EXISTS chat_rooms CASCADE;
            DROP TABLE IF EXISTS reviews CASCADE;
            DROP TABLE IF EXISTS escrow_trades CASCADE;
            DROP TABLE IF EXISTS product_tags CASCADE;
            DROP TABLE IF EXISTS product_images CASCADE;
            DROP TABLE IF EXISTS products CASCADE;
            DROP TABLE IF EXISTS wallet_transactions CASCADE;
            DROP TABLE IF EXISTS wallets CASCADE;
            DROP TABLE IF EXISTS users CASCADE;
        "#;
        manager.get_connection().execute_unprepared(sql).await?;
        Ok(())
    }
}
