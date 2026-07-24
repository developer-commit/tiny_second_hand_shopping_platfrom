// crates/backend/src/service/wallet_domain.rs
// 목적: 지갑 잔액 계산 및 에스크로 잠금 금액 로직 (단일 진실 공급원 - SSOT)

use crate::db::entity::{escrow_trade, wallet, wallet_transaction};
use crate::ports::wallet_port::EvmWalletPort;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::str::FromStr;
use std::sync::Arc;

pub struct BalanceInfo {
    pub onchain_eth_balance: f64,
    pub wallet_locked_amount_eth: f64,
    pub display_locked_amount: f64,
    pub available_eth_balance: f64,
}

pub struct WalletDomain;

impl WalletDomain {
    /// Pure function for calculating locked amounts and available balances
    pub fn calculate_balance_info(
        onchain_eth_balance: f64,
        escrows: &[(String, String, rust_decimal::Decimal)], // (currency, status, amount)
    ) -> BalanceInfo {
        let mut wallet_locked_amount_eth = Decimal::ZERO;
        let mut display_locked_amount = Decimal::ZERO;

        for (currency, status, amount) in escrows {
            let is_active = matches!(
                status.as_str(),
                "pending_deposit" | "deposited" | "received" | "disputed"
            );

            if is_active {
                display_locked_amount += amount;

                if currency == "ETH" {
                    // For ETH, only pending_deposit is still in the user's wallet
                    if status == "pending_deposit" {
                        wallet_locked_amount_eth += amount;
                    }
                }
            }
        }

        let wallet_locked_amount_eth_f64 =
            rust_decimal::prelude::ToPrimitive::to_f64(&wallet_locked_amount_eth).unwrap_or(0.0);
        let display_locked_amount_f64 =
            rust_decimal::prelude::ToPrimitive::to_f64(&display_locked_amount).unwrap_or(0.0);

        let available_eth_balance = onchain_eth_balance - wallet_locked_amount_eth_f64;

        BalanceInfo {
            onchain_eth_balance,
            wallet_locked_amount_eth: wallet_locked_amount_eth_f64,
            display_locked_amount: display_locked_amount_f64,
            available_eth_balance,
        }
    }

    /// 공통 지갑 잔액 및 잠금 금액 계산
    pub async fn get_balance_info<C: sea_orm::ConnectionTrait>(
        db: &C,
        wallet_port: Arc<dyn EvmWalletPort>,
        user_id: i64,
        eth_address: &str,
        wallet_id: i64,
    ) -> Result<BalanceInfo, String> {
        // 1. On-chain ETH Balance
        let mut onchain_eth_balance = 0.0;
        if let Ok(balance_wei) = wallet_port.get_balance(eth_address).await {
            if let Ok(eth_str) = ethers::utils::format_units(balance_wei, "ether") {
                onchain_eth_balance = eth_str.parse::<f64>().unwrap_or(0.0);
            }
        }

        // 2. Escrow Locked Amounts
        let escrows = escrow_trade::Entity::find()
            .filter(escrow_trade::Column::BuyerId.eq(user_id))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;

        let mapped_escrows: Vec<(String, String, rust_decimal::Decimal)> = escrows
            .into_iter()
            .map(|e| (e.currency, e.status, e.amount))
            .collect();

        Ok(Self::calculate_balance_info(
            onchain_eth_balance,
            &mapped_escrows,
        ))
    }

    /// 출금이나 에스크로 진행 전 잔액 검증
    pub async fn verify_sufficient_funds<C: sea_orm::ConnectionTrait>(
        db: &C,
        wallet_port: Arc<dyn EvmWalletPort>,
        user_id: i64,
        currency: &str,
        required_amount: f64,
        fee: f64,
    ) -> Result<(), String> {
        let wallet = wallet::Entity::find()
            .filter(wallet::Column::UserId.eq(user_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Wallet not found".to_string())?;

        let info = Self::get_balance_info(db, wallet_port, user_id, &wallet.eth_address, wallet.id)
            .await?;

        if currency == "ETH" {
            if info.available_eth_balance < (required_amount + fee) {
                return Err("Insufficient available funds for ETH".to_string());
            }
        } else {
            return Err(format!(
                "Unsupported currency for sufficient funds verification: {}",
                currency
            ));
        }

        Ok(())
    }
}
