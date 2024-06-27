use std::collections::BTreeMap;

use casper_engine_test_support::{ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR};
use casper_types::{contract_messages::Message, runtime_args, AddressableEntityHash, Key, U256};

use crate::utility::{
    constants::{
        AMOUNT, ARG_DECIMALS, ARG_NAME, ARG_SYMBOL, ARG_TOTAL_SUPPLY, CEP18_TOKEN_CONTRACT_KEY,
        ENABLE_MINT_BURN, EVENTS, EVENTS_MODE, METHOD_MINT, OWNER, TOKEN_DECIMALS, TOKEN_NAME,
        TOKEN_OWNER_ADDRESS_1, TOKEN_OWNER_ADDRESS_1_OLD, TOKEN_OWNER_AMOUNT_1, TOKEN_SYMBOL,
        TOKEN_TOTAL_SUPPLY,
    },
    installer_request_builders::{setup_with_args, TestContext},
    message_handlers::{entity, message_topic},
};

#[test]
fn should_have_native_events() {
    let (mut builder, TestContext { cep18_token, .. }) = setup_with_args(runtime_args! {
        ARG_NAME => TOKEN_NAME,
        ARG_SYMBOL => TOKEN_SYMBOL,
        ARG_DECIMALS => TOKEN_DECIMALS,
        ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
        EVENTS_MODE => 2_u8,
        ENABLE_MINT_BURN => true,
    });

    let addressable_cep18_token = AddressableEntityHash::new(cep18_token.value());
    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        addressable_cep18_token,
        METHOD_MINT,
        runtime_args! {OWNER => TOKEN_OWNER_ADDRESS_1, AMOUNT => U256::from(TOKEN_OWNER_AMOUNT_1)},
    )
    .build();
    builder.exec(mint_request).expect_success().commit();

    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap();
    let account_named_keys = account.named_keys();

    let cep18_token = account_named_keys
        .get(CEP18_TOKEN_CONTRACT_KEY)
        .and_then(|key| key.into_entity_hash())
        .expect("should have contract hash");

    // events check
    let entity = entity(&builder, &cep18_token);

    let (topic_name, message_topic_hash) = entity
        .message_topics()
        .iter()
        .next()
        .expect("should have at least one topic");

    let mut user_map: BTreeMap<Key, bool> = BTreeMap::new();
    user_map.insert(Key::Account(*DEFAULT_ACCOUNT_ADDR), true);
    user_map.insert(TOKEN_OWNER_ADDRESS_1_OLD, true);

    assert_eq!(topic_name, &EVENTS.to_string());

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        METHOD_MINT,
        runtime_args! {OWNER => TOKEN_OWNER_ADDRESS_1, AMOUNT => U256::from(TOKEN_OWNER_AMOUNT_1)},
    )
    .build();

    builder.exec(mint_request).expect_success().commit();

    assert_eq!(
        message_topic(&builder, &cep18_token, *message_topic_hash).message_count(),
        3
    );

    let exec_result = builder.get_exec_result_owned(3).unwrap();
    let messages = exec_result.messages();
    let mint_message = "Mint(Mint { recipient: Key::AddressableEntity(account-2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a), amount: 1000000 })";
    let message = Message::new(
        casper_types::EntityAddr::SmartContract(cep18_token.value()),
        mint_message.into(),
        EVENTS.to_string(),
        *message_topic_hash,
        2,
        2,
    );
    assert_eq!(messages, &vec![message]);
}
