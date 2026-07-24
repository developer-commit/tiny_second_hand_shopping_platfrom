// crates/shared/src/dto/transaction_dto.rs
// 목적: 지갑 잔액 조회 및 트랜잭션 내역 DTO.
// 스키마 은닉:
// - balance → available_balance, tx_type → movement_type
// - network_fee → fee_deducted, tx_hash → blockchain_hash
// - status → process_status
// 보안: 지갑 개인키, 니모닉은 절대 DTO에 포함되지 않습니다.

use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::types::OpaqueId;

/// 트랜잭션 유형 Enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MovementType {
    Deposit,    // DB: "deposit"
    Withdrawal, // DB: "withdrawal"
    EscrowLock, // 에스크로 잠금
    EscrowRelease, // 에스크로 해제
}

/// 트랜잭션 처리 상태 Enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TxStatus {
    Pending,   // DB: "pending"
    Confirmed, // DB: "confirmed"
    Failed,    // DB: "failed"
}

/// [Response] GET /wallet — 지갑 상태 조회
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletStateRes {
    pub public_address: String,      // BCH 입금용 CashAddr 주소 (공개)
    pub available_balance: f64,      // DB: balance
    pub locked_in_escrow: f64,       // 에스크로로 묶인 금액 (계산값)
    #[serde(default)]
    pub eth_balance: f64,            // ETH 잔액
}



/// [Response] 트랜잭션 내역 항목
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxHistoryItemRes {
    pub tx_uid: OpaqueId,            // DB: id (난독화)
    pub movement_type: MovementType, // DB: tx_type
    pub amount: f64,
    pub fee_deducted: f64,           // DB: network_fee
    pub blockchain_hash: Option<String>, // DB: tx_hash
    pub process_status: TxStatus,    // DB: status
    pub timestamp: String,           // DB: created_at
}

/// [Request] POST /wallet/eth/withdraw — ETH 출금 요청
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct EthWithdrawReq {
    #[validate(length(min = 40, max = 42))]
    pub destination_eth: String,     // 외부 ETH 주소
    #[validate(range(min = 0.000_01))]
    pub amount_eth: f64,
    #[validate(length(min = 6, max = 8))]
    pub otp_token: String,           // 2FA OTP 코드
}


