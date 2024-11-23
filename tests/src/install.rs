use casper_engine_test_support::DEFAULT_ACCOUNT_ADDR;
use casper_types::{Key, U256};
use cowl_cep18::constants::{
    ARG_DECIMALS, ARG_NAME, ARG_SYMBOL, ARG_TOTAL_SUPPLY, DICT_ALLOWANCES, DICT_BALANCES,
};

use crate::utility::{
    constants::{TOKEN_DECIMALS, TOKEN_NAME, TOKEN_SYMBOL, TOKEN_TOTAL_SUPPLY},
    installer_request_builders::{
        cep18_check_balance_of, invert_cep18_address, setup, TestContext,
    },
};

#[test]
fn should_have_queryable_properties() {
    let (mut builder, TestContext { cep18_token, .. }) = setup();

    let name: String = builder.get_value(cep18_token, ARG_NAME);
    assert_eq!(name, TOKEN_NAME);

    let symbol: String = builder.get_value(cep18_token, ARG_SYMBOL);
    assert_eq!(symbol, TOKEN_SYMBOL);

    let decimals: u8 = builder.get_value(cep18_token, ARG_DECIMALS);
    assert_eq!(decimals, TOKEN_DECIMALS);

    let total_supply: U256 = builder.get_value(cep18_token, ARG_TOTAL_SUPPLY);
    assert_eq!(total_supply, U256::from(TOKEN_TOTAL_SUPPLY));

    let owner_key = Key::Account(*DEFAULT_ACCOUNT_ADDR);

    let owner_balance = cep18_check_balance_of(&mut builder, &cep18_token, owner_key);
    assert_eq!(owner_balance, total_supply);

    let contract_balance =
        cep18_check_balance_of(&mut builder, &cep18_token, Key::Hash(cep18_token.value()));
    assert_eq!(contract_balance, U256::zero());

    // Ensures that Account and Contract ownership is respected and we're not keying ownership under
    // the raw bytes regardless of variant.
    let inverted_owner_key = invert_cep18_address(owner_key);
    let inverted_owner_balance =
        cep18_check_balance_of(&mut builder, &cep18_token, inverted_owner_key);
    assert_eq!(inverted_owner_balance, U256::zero());
}

#[test]
fn should_not_store_balances_or_allowances_under_account_after_install() {
    let (builder, _contract_hash) = setup();

    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let named_keys = account.named_keys();
    assert!(!named_keys.contains_key(DICT_BALANCES), "{:?}", named_keys);
    assert!(
        !named_keys.contains_key(DICT_ALLOWANCES),
        "{:?}",
        named_keys
    );
}
