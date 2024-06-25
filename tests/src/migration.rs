use std::collections::BTreeMap;

use casper_engine_test_support::{
    ExecuteRequestBuilder, UpgradeRequestBuilder, DEFAULT_ACCOUNT_ADDR,
};
use casper_types::{runtime_args, EraId, Key, ProtocolVersion, RuntimeArgs, U256};

use crate::utility::{
    constants::{
        AMOUNT, ARG_DECIMALS, ARG_NAME, ARG_SYMBOL, ARG_TOTAL_SUPPLY, CEP18_CONTRACT_WASM,
        CEP18_TEST_CONTRACT_WASM, CEP18_TOKEN_CONTRACT_KEY, EVENTS, EVENTS_MODE, METHOD_MINT,
        MIGRATE_USER_BALANCE_KEYS_ENTRY_POINT_NAME, MIGRATE_USER_SEC_KEYS_ENTRY_POINT_NAME, OWNER,
        REVERT, TOKEN_DECIMALS, TOKEN_NAME, TOKEN_OWNER_ADDRESS_1, TOKEN_OWNER_ADDRESS_1_OLD,
        TOKEN_OWNER_AMOUNT_1, TOKEN_SYMBOL, TOKEN_TOTAL_SUPPLY, USER_KEY_MAP,
    },
    installer_request_builders::{cep18_check_balance_of, setup, TestContext},
};

#[test]
fn should_upgrade_contract_version() {
    let (mut builder, TestContext { cep18_contract_hash: _, .. }) = setup();

    let version_0: u32 = builder
        .query(
            None,
            Key::Account(*DEFAULT_ACCOUNT_ADDR),
            &["cep18_contract_version_CasperTest".to_string()],
        )
        .unwrap()
        .as_cl_value()
        .unwrap()
        .clone()
        .into_t()
        .unwrap();

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CEP18_CONTRACT_WASM,
        runtime_args! {
            ARG_NAME => TOKEN_NAME,
            ARG_SYMBOL => TOKEN_SYMBOL,
            ARG_DECIMALS => TOKEN_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let version_1: u32 = builder
        .query(
            None,
            Key::Account(*DEFAULT_ACCOUNT_ADDR),
            &["cep18_contract_version_CasperTest".to_string()],
        )
        .unwrap()
        .as_cl_value()
        .unwrap()
        .clone()
        .into_t()
        .unwrap();

    assert!(version_0 < version_1);
}

#[test]
fn should_migrate_1_5_6_to_2_0_0_rc3() {
    let (mut builder, lmdb_fixture_state, _temp_dir) =
        casper_fixtures::builder_from_global_state_fixture("cep18-1.5.6-minted");

    assert_eq!(
        builder.get_post_state_hash(),
        lmdb_fixture_state.post_state_hash
    );

    let mut upgrade_config = UpgradeRequestBuilder::new()
        .with_current_protocol_version(lmdb_fixture_state.genesis_protocol_version())
        .with_new_protocol_version(ProtocolVersion::V2_0_0)
        .with_migrate_legacy_accounts(true)
        .with_migrate_legacy_contracts(true)
        .with_activation_point(EraId::new(1))
        .build();

    builder
        .upgrade(&mut upgrade_config)
        .expect_upgrade_success()
        .commit();
    assert_ne!(
        builder.get_post_state_hash(),
        lmdb_fixture_state.post_state_hash
    );

    let version_0: u32 = builder
        .query(
            None,
            Key::Account(*DEFAULT_ACCOUNT_ADDR),
            &["cep18_contract_version_CasperTest".to_string()],
        )
        .unwrap()
        .as_cl_value()
        .unwrap()
        .clone()
        .into_t()
        .unwrap();

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CEP18_CONTRACT_WASM,
        runtime_args! {
            ARG_NAME => TOKEN_NAME,
            ARG_SYMBOL => TOKEN_SYMBOL,
            ARG_DECIMALS => TOKEN_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
            EVENTS_MODE => 3_u8
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let version_1: u32 = builder
        .query(
            None,
            Key::Account(*DEFAULT_ACCOUNT_ADDR),
            &["cep18_contract_version_CasperTest".to_string()],
        )
        .unwrap()
        .as_cl_value()
        .unwrap()
        .clone()
        .into_t()
        .unwrap();

    assert!(version_0 < version_1);

    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap();
    let account_named_keys = account.named_keys();

    let cep18_contract_hash = account_named_keys
        .get(CEP18_TOKEN_CONTRACT_KEY)
        .and_then(|key| key.into_entity_hash())
        .expect("should have contract hash");

    let mut user_map: BTreeMap<Key, bool> = BTreeMap::new();
    user_map.insert(Key::Account(*DEFAULT_ACCOUNT_ADDR), true);
    user_map.insert(TOKEN_OWNER_ADDRESS_1_OLD, true);

    let sec_key_migrate_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_contract_hash,
        MIGRATE_USER_SEC_KEYS_ENTRY_POINT_NAME,
        runtime_args! {EVENTS => true, REVERT => true, USER_KEY_MAP => &user_map},
    )
    .build();

    builder
        .exec(sec_key_migrate_request)
        .expect_success()
        .commit();

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_contract_hash,
        METHOD_MINT,
        runtime_args! {OWNER => TOKEN_OWNER_ADDRESS_1, AMOUNT => U256::from(TOKEN_OWNER_AMOUNT_1)},
    )
    .build();

    builder.exec(mint_request).expect_success().commit();

    let test_contract = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CEP18_TEST_CONTRACT_WASM,
        RuntimeArgs::default(),
    )
    .build();

    builder.exec(test_contract).expect_success().commit();

    assert_eq!(
        cep18_check_balance_of(&mut builder, &cep18_contract_hash, TOKEN_OWNER_ADDRESS_1),
        U256::from(TOKEN_OWNER_AMOUNT_1),
    );

    let balance_migrate_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_contract_hash,
        MIGRATE_USER_BALANCE_KEYS_ENTRY_POINT_NAME,
        runtime_args! {EVENTS => true, REVERT => true, USER_KEY_MAP => user_map},
    )
    .build();

    builder
        .exec(balance_migrate_request)
        .expect_success()
        .commit();

    // even if we minted before migrating, the balance should persist
    assert_eq!(
        cep18_check_balance_of(&mut builder, &cep18_contract_hash, TOKEN_OWNER_ADDRESS_1),
        U256::from(TOKEN_OWNER_AMOUNT_1 * 2),
    );
}
