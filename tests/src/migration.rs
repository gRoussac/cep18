use casper_engine_test_support::{ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR};
use casper_types::{runtime_args, Key, U256};

use crate::utility::{
    constants::{
        ARG_DECIMALS, ARG_NAME, ARG_SYMBOL, ARG_TOTAL_SUPPLY, CEP18_CONTRACT_WASM,
        CEP18_TOKEN_CONTRACT_KEY, TOKEN_DECIMALS, TOKEN_NAME, TOKEN_SYMBOL, TOKEN_TOTAL_SUPPLY,
    },
    installer_request_builders::{setup, TestContext},
};

#[test]
fn should_upgrade_contract_version() {
    let (mut builder, TestContext { cep18_token: _, .. }) = setup();

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
fn should_migrate_1_5_6_to_2_0_0_rc2() {
    let (mut builder, lmdb_fixture_state, _) =
        casper_fixtures::builder_from_global_state_fixture("cep18-1.5.6-minted");

    let get_entity_by_account_hash = builder.get_entity_by_account_hash(*DEFAULT_ACCOUNT_ADDR);
    println!("{get_entity_by_account_hash:?}");

    let query_named_key_by_account_hash = builder.query_named_key_by_account_hash(
        Some(lmdb_fixture_state.post_state_hash),
        *DEFAULT_ACCOUNT_ADDR,
        CEP18_TOKEN_CONTRACT_KEY,
    );
    println!("{query_named_key_by_account_hash:?}");

    let get_entity_hash_by_account_hash =
        builder.get_entity_hash_by_account_hash(*DEFAULT_ACCOUNT_ADDR);
    println!("{get_entity_hash_by_account_hash:?}");

    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap();
    let account_named_keys = account.named_keys();

    let _cep18_token = account_named_keys
        .get(CEP18_TOKEN_CONTRACT_KEY)
        .and_then(|key| key.into_entity_hash())
        .expect("should have contract hash");

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
