use casper_engine_test_support::DEFAULT_ACCOUNT_ADDR;
use casper_types::{EntityAddr, Key, U256};

use crate::utility::{
    constants::{
        ALLOWANCES_KEY, BALANCES_KEY, DECIMALS_KEY, NAME_KEY, SYMBOL_KEY, TOKEN_DECIMALS,
        TOKEN_NAME, TOKEN_SYMBOL, TOKEN_TOTAL_SUPPLY, TOTAL_SUPPLY_KEY,
    },
    installer_request_builders::{
        cep18_check_balance_of, invert_cep18_address, setup, TestContext,
    },
};

#[test]
fn should_have_queryable_properties() {
    let (mut builder, TestContext { cep18_contract_hash, .. }) = setup();

    let cep18_entity_addr = EntityAddr::new_smart_contract(cep18_contract_hash.value());

    let name: String = builder.get_value(cep18_entity_addr, NAME_KEY);
    assert_eq!(name, TOKEN_NAME);

    let symbol: String = builder.get_value(cep18_entity_addr, SYMBOL_KEY);
    assert_eq!(symbol, TOKEN_SYMBOL);

    let decimals: u8 = builder.get_value(cep18_entity_addr, DECIMALS_KEY);
    assert_eq!(decimals, TOKEN_DECIMALS);

    let total_supply: U256 = builder.get_value(cep18_entity_addr, TOTAL_SUPPLY_KEY);
    assert_eq!(total_supply, U256::from(TOKEN_TOTAL_SUPPLY));

    let owner_key = Key::AddressableEntity(EntityAddr::Account(DEFAULT_ACCOUNT_ADDR.value()));

    let owner_balance = cep18_check_balance_of(&mut builder, &cep18_contract_hash, owner_key);
    assert_eq!(owner_balance, total_supply);

    let contract_balance =
        cep18_check_balance_of(&mut builder, &cep18_contract_hash, Key::Hash(cep18_contract_hash.value()));
    assert_eq!(contract_balance, U256::zero());

    // Ensures that Account and Contract ownership is respected and we're not keying ownership under
    // the raw bytes regardless of variant.
    let inverted_owner_key = invert_cep18_address(owner_key);
    let inverted_owner_balance =
        cep18_check_balance_of(&mut builder, &cep18_contract_hash, inverted_owner_key);
    assert_eq!(inverted_owner_balance, U256::zero());
}

#[test]
fn should_not_store_balances_or_allowances_under_account_after_install() {
    let (builder, _contract_hash) = setup();

    let named_keys = builder.get_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR);

    assert!(!named_keys.contains(BALANCES_KEY), "{:?}", named_keys);
    assert!(!named_keys.contains(ALLOWANCES_KEY), "{:?}", named_keys);
}
