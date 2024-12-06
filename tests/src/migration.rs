use casper_engine_test_support::{ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR};
use casper_types::{runtime_args, Key, RuntimeArgs};
use cowl_cep18::{
    constants::{ARG_NAME, ARG_UPGRADE_FLAG},
    events::Upgrade,
};

use crate::utility::{
    constants::{CEP18_CONTRACT_WASM, CEP18_TEST_TOKEN_CONTRACT_VERSION, TOKEN_NAME},
    installer_request_builders::{setup, TestContext},
    support::get_event,
};

#[test]
fn should_upgrade_contract_version() {
    let (mut builder, TestContext { cep18_token, .. }) = setup();

    let version_0: u32 = builder
        .query(
            None,
            Key::Account(*DEFAULT_ACCOUNT_ADDR),
            &[CEP18_TEST_TOKEN_CONTRACT_VERSION.to_string()],
        )
        .unwrap()
        .as_cl_value()
        .unwrap()
        .clone()
        .into_t()
        .unwrap();

    /* COWL */
    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CEP18_CONTRACT_WASM,
        runtime_args! {
            ARG_NAME => TOKEN_NAME,
            ARG_UPGRADE_FLAG => true,
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let version_1: u32 = builder
        .query(
            None,
            Key::Account(*DEFAULT_ACCOUNT_ADDR),
            &[CEP18_TEST_TOKEN_CONTRACT_VERSION.to_string()],
        )
        .unwrap()
        .as_cl_value()
        .unwrap()
        .clone()
        .into_t()
        .unwrap();

    assert!(version_0 < version_1);

    // Expect Upgrade event
    let expected_event = Upgrade::new();
    let event_index = 1; // Mint + Upgrade
    let actual_event: Upgrade = get_event(&builder, &cep18_token.into(), event_index);
    assert_eq!(actual_event, expected_event, "Expected Upgrade event.");
    /*  */
}
