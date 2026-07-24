use crate::service::wallet_domain::WalletDomain;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

#[test]
fn test_eth_escrow_lifecycle() {
    let onchain_balance = 100.0;

    // Scenario 1: pending_deposit (ETH)
    let escrows = vec![(
        "ETH".to_string(),
        "pending_deposit".to_string(),
        Decimal::from_f64(10.0).unwrap(),
    )];
    let info = WalletDomain::calculate_balance_info(onchain_balance, &escrows);

    assert_eq!(info.wallet_locked_amount_eth, 10.0);
    assert_eq!(info.display_locked_amount, 10.0);
    assert_eq!(info.available_eth_balance, 90.0); // 100 - 10

    // Scenario 2: deposited (ETH)
    let escrows_deposited = vec![(
        "ETH".to_string(),
        "deposited".to_string(),
        Decimal::from_f64(10.0).unwrap(),
    )];
    // Onchain balance drops after deposit
    let onchain_balance_after = 90.0;
    let info2 = WalletDomain::calculate_balance_info(onchain_balance_after, &escrows_deposited);

    assert_eq!(info2.wallet_locked_amount_eth, 0.0); // deposited no longer locked in DB wallet
    assert_eq!(info2.display_locked_amount, 10.0);
    assert_eq!(info2.available_eth_balance, 90.0); // 90 - 0

    // Scenario 3: received and disputed (ETH)
    let escrows_misc = vec![
        (
            "ETH".to_string(),
            "received".to_string(),
            Decimal::from_f64(5.0).unwrap(),
        ),
        (
            "ETH".to_string(),
            "disputed".to_string(),
            Decimal::from_f64(5.0).unwrap(),
        ),
    ];
    let info3 = WalletDomain::calculate_balance_info(80.0, &escrows_misc);

    assert_eq!(info3.wallet_locked_amount_eth, 0.0);
    assert_eq!(info3.display_locked_amount, 10.0);
    assert_eq!(info3.available_eth_balance, 80.0);
}

#[test]
fn test_mixed_currencies() {
    let escrows = vec![
        (
            "ETH".to_string(),
            "pending_deposit".to_string(),
            Decimal::from_f64(10.0).unwrap(),
        ),
        (
            "ETH".to_string(),
            "deposited".to_string(),
            Decimal::from_f64(10.0).unwrap(),
        ),
        (
            "UNSUPPORTED".to_string(),
            "deposited".to_string(),
            Decimal::from_f64(20.0).unwrap(),
        ),
    ];

    let info = WalletDomain::calculate_balance_info(100.0, &escrows);

    // UNSUPPORTED is ignored for wallet_locked_amount_eth, but included in display_locked_amount
    assert_eq!(info.wallet_locked_amount_eth, 10.0);
    assert_eq!(info.display_locked_amount, 40.0);
    assert_eq!(info.available_eth_balance, 90.0);
}
