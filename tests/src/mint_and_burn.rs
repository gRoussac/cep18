use std::collections::HashMap;

use casper_engine_test_support::{ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR};
use casper_types::{runtime_args, ApiError, Key, RuntimeArgs, U256};
use cowl_cep18::constants::{
    ADMIN_LIST, ARG_AMOUNT, ARG_DECIMALS, ARG_ENABLE_MINT_BURN, ARG_NAME, ARG_OWNER, ARG_SYMBOL,
    ARG_TOTAL_SUPPLY, ENTRY_POINT_BURN, ENTRY_POINT_CHANGE_SECURITY, ENTRY_POINT_MINT, MINTER_LIST,
    NONE_LIST,
};

use crate::utility::{
    constants::{
        ACCOUNT_USER_1, ACCOUNT_USER_2, TOKEN_DECIMALS, TOKEN_NAME, TOKEN_OWNER_AMOUNT_1,
        TOKEN_OWNER_AMOUNT_2, TOKEN_SYMBOL, TOKEN_TOTAL_SUPPLY,
    },
    installer_request_builders::{
        cep18_check_balance_of, cep18_check_total_supply, setup_with_args, TestContext,
    },
    support::{create_dummy_key_pair, fund_account},
};

use casper_execution_engine::core::{
    engine_state::Error as CoreError, execution::Error as ExecError,
};

#[test]
fn test_mint_and_burn_tokens() {
    let mint_amount = U256::one();

    let (
        mut builder,
        TestContext {
            cep18_token,
            ref test_accounts,
            ..
        },
    ) = setup_with_args(
        runtime_args! {
            ARG_NAME => TOKEN_NAME,
            ARG_SYMBOL => TOKEN_SYMBOL,
            ARG_DECIMALS => TOKEN_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
            ARG_ENABLE_MINT_BURN => true,
        },
        None,
    );
    let account_user_1 = *test_accounts.get(&ACCOUNT_USER_1).unwrap();
    let account_user_1_key = Key::Account(account_user_1);
    let account_user_2 = *test_accounts.get(&ACCOUNT_USER_2).unwrap();
    let account_user_2_key = Key::Account(account_user_2);

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        ENTRY_POINT_MINT,
        runtime_args! {ARG_OWNER => account_user_1_key, ARG_AMOUNT => U256::from(TOKEN_OWNER_AMOUNT_1)},
    )
    .build();
    builder.exec(mint_request).expect_success().commit();
    let mint_request_2 = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        ENTRY_POINT_MINT,
        runtime_args! {ARG_OWNER => account_user_2_key, ARG_AMOUNT => U256::from(TOKEN_OWNER_AMOUNT_2)},
    )
    .build();
    builder.exec(mint_request_2).expect_success().commit();
    assert_eq!(
        cep18_check_balance_of(
            &mut builder,
            &cep18_token,
            Key::Account(*DEFAULT_ACCOUNT_ADDR)
        ),
        U256::from(TOKEN_TOTAL_SUPPLY),
    );
    assert_eq!(
        cep18_check_balance_of(&mut builder, &cep18_token, account_user_1_key),
        U256::from(TOKEN_OWNER_AMOUNT_1)
    );
    assert_eq!(
        cep18_check_balance_of(&mut builder, &cep18_token, account_user_2_key),
        U256::from(TOKEN_OWNER_AMOUNT_2)
    );
    let total_supply_before_mint = cep18_check_total_supply(&mut builder, &cep18_token);

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_OWNER => account_user_1_key,
            ARG_AMOUNT => mint_amount,
        },
    )
    .build();

    builder.exec(mint_request).expect_success().commit();

    assert_eq!(
        cep18_check_balance_of(&mut builder, &cep18_token, account_user_1_key),
        U256::from(TOKEN_OWNER_AMOUNT_1) + mint_amount,
    );
    assert_eq!(
        cep18_check_balance_of(&mut builder, &cep18_token, account_user_2_key),
        U256::from(TOKEN_OWNER_AMOUNT_2)
    );

    let total_supply_after_mint = cep18_check_total_supply(&mut builder, &cep18_token);
    assert_eq!(
        total_supply_after_mint,
        total_supply_before_mint + mint_amount,
    );
    let total_supply_before_burn = total_supply_after_mint;

    let burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_AMOUNT => mint_amount,
        },
    )
    .build();

    builder.exec(burn_request).expect_success().commit();

    assert_eq!(
        cep18_check_balance_of(
            &mut builder,
            &cep18_token,
            Key::Account(*DEFAULT_ACCOUNT_ADDR)
        ),
        U256::from(999999999),
    );
    assert_eq!(
        cep18_check_balance_of(&mut builder, &cep18_token, account_user_2_key),
        U256::from(TOKEN_OWNER_AMOUNT_2)
    );
    let total_supply_after_burn = cep18_check_total_supply(&mut builder, &cep18_token);
    assert_eq!(
        total_supply_after_burn,
        total_supply_before_burn - mint_amount,
    );

    assert_eq!(total_supply_after_burn, total_supply_before_mint);
}

#[test]
fn test_should_not_mint_above_limits() {
    let mint_amount = U256::MAX;

    let (
        mut builder,
        TestContext {
            cep18_token,
            ref test_accounts,
            ..
        },
    ) = setup_with_args(
        runtime_args! {
            ARG_NAME => TOKEN_NAME,
            ARG_SYMBOL => TOKEN_SYMBOL,
            ARG_DECIMALS => TOKEN_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
            "enable_mint_burn" => true,
        },
        None,
    );

    let account_user_1 = *test_accounts.get(&ACCOUNT_USER_1).unwrap();
    let account_user_1_key = Key::Account(account_user_1);
    let account_user_2 = *test_accounts.get(&ACCOUNT_USER_2).unwrap();
    let account_user_2_key = Key::Account(account_user_2);

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        ENTRY_POINT_MINT,
        runtime_args! {ARG_OWNER => account_user_1_key, ARG_AMOUNT => U256::from(TOKEN_OWNER_AMOUNT_1)},
    )
    .build();
    builder.exec(mint_request).expect_success().commit();
    let mint_request_2 = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        ENTRY_POINT_MINT,
        runtime_args! {ARG_OWNER => account_user_2_key, ARG_AMOUNT => U256::from(TOKEN_OWNER_AMOUNT_2)},
    )
    .build();
    builder.exec(mint_request_2).expect_success().commit();
    assert_eq!(
        cep18_check_balance_of(&mut builder, &cep18_token, account_user_1_key),
        U256::from(TOKEN_OWNER_AMOUNT_1)
    );

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_OWNER => account_user_1_key,
            ARG_AMOUNT => mint_amount,
        },
    )
    .build();

    builder.exec(mint_request).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == cowl_cep18::error::Cep18Error::Overflow as u16),
        "{:?}",
        error
    );
}

#[test]
fn test_should_not_burn_above_balance() {
    let (mut builder, TestContext { cep18_token, .. }) = setup_with_args(
        runtime_args! {
            ARG_NAME => TOKEN_NAME,
            ARG_SYMBOL => TOKEN_SYMBOL,
            ARG_DECIMALS => TOKEN_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
            "enable_mint_burn" => true,
        },
        None,
    );

    let burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_AMOUNT => U256::from(TOKEN_TOTAL_SUPPLY)+1,
        },
    )
    .build();

    builder.exec(burn_request).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == cowl_cep18::error::Cep18Error::InsufficientBalance as u16),
        "{:?}",
        error
    );
}

#[test]
fn test_should_not_mint_or_burn_with_entrypoint_disabled() {
    let mint_amount = U256::one();

    let (
        mut builder,
        TestContext {
            cep18_token,
            ref test_accounts,
            ..
        },
    ) = setup_with_args(
        runtime_args! {
            ARG_NAME => TOKEN_NAME,
            ARG_SYMBOL => TOKEN_SYMBOL,
            ARG_DECIMALS => TOKEN_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
            ARG_ENABLE_MINT_BURN => false,
        },
        None,
    );

    let account_user_1 = *test_accounts.get(&ACCOUNT_USER_1).unwrap();
    let account_user_1_key = Key::Account(account_user_1);

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_OWNER => account_user_1_key,
            ARG_AMOUNT => mint_amount,
        },
    )
    .build();

    builder.exec(mint_request).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == 60016),
        "{:?}",
        error
    );

    let burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_OWNER => account_user_1_key,
            ARG_AMOUNT => mint_amount,
        },
    )
    .build();

    builder.exec(burn_request).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == 60016),
        "{:?}",
        error
    );
}

#[test]
fn test_security_no_rights() {
    let mint_amount = U256::one();

    let (
        mut builder,
        TestContext {
            cep18_token,
            ref test_accounts,
            ..
        },
    ) = setup_with_args(
        runtime_args! {
            ARG_NAME => TOKEN_NAME,
            ARG_SYMBOL => TOKEN_SYMBOL,
            ARG_DECIMALS => TOKEN_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
            ARG_ENABLE_MINT_BURN => true,
        },
        None,
    );

    let account_user_1 = *test_accounts.get(&ACCOUNT_USER_1).unwrap();

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        cep18_token,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_OWNER => Key::Account(account_user_1),
            ARG_AMOUNT => mint_amount,
        },
    )
    .build();

    builder.exec(mint_request).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == 60010),
        "{:?}",
        error
    );

    let passing_admin_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_OWNER => Key::Account(account_user_1),
            ARG_AMOUNT => mint_amount,
        },
    )
    .build();

    builder
        .exec(passing_admin_mint_request)
        .expect_success()
        .commit();

    let burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        cep18_token,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_OWNER => Key::Account(account_user_1),
            ARG_AMOUNT => mint_amount,
        },
    )
    .build();

    builder.exec(burn_request).expect_success().commit();
}

#[test]
fn test_security_minter_rights() {
    let (_, public_key_account_user_1) = create_dummy_key_pair(ACCOUNT_USER_1);
    let account_user_1 = public_key_account_user_1.to_account_hash();
    let mut test_accounts = HashMap::new();
    test_accounts.insert(ACCOUNT_USER_1, account_user_1);

    let mint_amount = U256::one();

    let (
        mut builder,
        TestContext {
            cep18_token,
            ref test_accounts,
            ..
        },
    ) = setup_with_args(
        runtime_args! {
            ARG_NAME => TOKEN_NAME,
            ARG_SYMBOL => TOKEN_SYMBOL,
            ARG_DECIMALS => TOKEN_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
            ARG_ENABLE_MINT_BURN => true,
            MINTER_LIST => vec![Key::Account(account_user_1)]
        },
        Some(test_accounts),
    );

    let account_user_1 = *test_accounts.get(&ACCOUNT_USER_1).unwrap();
    let account_user_1_key = Key::Account(account_user_1);

    // account_user_1 was created before genesis and is not yet funded so fund it
    fund_account(&mut builder, account_user_1);

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        cep18_token,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_OWNER => account_user_1_key,
            ARG_AMOUNT => mint_amount,
        },
    )
    .build();

    builder.exec(mint_request).commit().expect_success();
}

#[test]
fn test_security_burner_rights() {
    let mint_amount = U256::one();

    let (
        mut builder,
        TestContext {
            cep18_token,
            ref test_accounts,
            ..
        },
    ) = setup_with_args(
        runtime_args! {
            ARG_NAME => TOKEN_NAME,
            ARG_SYMBOL => TOKEN_SYMBOL,
            ARG_DECIMALS => TOKEN_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
            ARG_ENABLE_MINT_BURN => true,
        },
        None,
    );

    let account_user_1 = *test_accounts.get(&ACCOUNT_USER_1).unwrap();
    let account_user_1_key = Key::Account(account_user_1);

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        cep18_token,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_OWNER => account_user_1_key,
            ARG_AMOUNT => mint_amount,
        },
    )
    .build();

    builder.exec(mint_request).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == 60010),
        "{:?}",
        error
    );

    // mint by admin
    let working_mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_AMOUNT => mint_amount,
        },
    )
    .build();

    builder.exec(working_mint_request).commit().expect_success();

    // any user can burn
    let burn_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        ENTRY_POINT_BURN,
        runtime_args! {
            ARG_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_AMOUNT => mint_amount,
        },
    )
    .build();

    builder.exec(burn_request).commit().expect_success();
}

#[test]
fn test_change_security() {
    let (_, public_key_account_user_1) = create_dummy_key_pair(ACCOUNT_USER_1);
    let account_user_1 = public_key_account_user_1.to_account_hash();
    let mut test_accounts = HashMap::new();
    test_accounts.insert(ACCOUNT_USER_1, account_user_1);

    let mint_amount = U256::one();

    let (mut builder, TestContext { cep18_token, .. }) = setup_with_args(
        runtime_args! {
            ARG_NAME => TOKEN_NAME,
            ARG_SYMBOL => TOKEN_SYMBOL,
            ARG_DECIMALS => TOKEN_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
            ARG_ENABLE_MINT_BURN => true,
            ADMIN_LIST => vec![Key::Account(account_user_1)]
        },
        Some(test_accounts),
    );

    // account_user_1 was created before genesis and is not yet funded so fund it
    fund_account(&mut builder, account_user_1);

    let account_user_1_key = Key::Account(account_user_1);

    let change_security_request = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_1,
        cep18_token,
        ENTRY_POINT_CHANGE_SECURITY,
        runtime_args! {
            NONE_LIST => vec![Key::Account(*DEFAULT_ACCOUNT_ADDR)],
        },
    )
    .build();

    builder
        .exec(change_security_request)
        .commit()
        .expect_success();

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_OWNER => account_user_1_key,
            ARG_AMOUNT => mint_amount,
        },
    )
    .build();

    builder.exec(mint_request).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == 60010),
        "{:?}",
        error
    );
}
