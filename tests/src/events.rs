use casper_engine_test_support::{ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR};
use casper_types::{
    addressable_entity::EntityKindTag, bytesrepr::Bytes, contract_messages::Message, runtime_args,
    AddressableEntityHash, Key, U256,
};

use crate::utility::{
    constants::{
        AMOUNT, ARG_DECIMALS, ARG_NAME, ARG_SYMBOL, ARG_TOTAL_SUPPLY, CEP18_TOKEN_CONTRACT_KEY,
        ENABLE_MINT_BURN, EVENTS, EVENTS_MODE, METHOD_MINT, OWNER, TOKEN_DECIMALS, TOKEN_NAME,
        TOKEN_OWNER_ADDRESS_1, TOKEN_OWNER_AMOUNT_1, TOKEN_SYMBOL, TOKEN_TOTAL_SUPPLY,
    },
    installer_request_builders::{setup_with_args, TestContext},
    message_handlers::message_topic,
    support::get_dictionary_value_from_key,
};

#[test]
fn should_have_have_no_events() {
    let (
        mut builder,
        TestContext {
            cep18_contract_hash,
            ..
        },
    ) = setup_with_args(runtime_args! {
        ARG_NAME => TOKEN_NAME,
        ARG_SYMBOL => TOKEN_SYMBOL,
        ARG_DECIMALS => TOKEN_DECIMALS,
        ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
        EVENTS_MODE => 0_u8,
        ENABLE_MINT_BURN => true,
    });

    let addressable_cep18_token = AddressableEntityHash::new(cep18_contract_hash.value());
    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        addressable_cep18_token,
        METHOD_MINT,
        runtime_args! {OWNER => TOKEN_OWNER_ADDRESS_1, AMOUNT => U256::from(TOKEN_OWNER_AMOUNT_1)},
    )
    .build();
    builder.exec(mint_request).expect_success().commit();

    let entity_with_named_keys = builder.get_named_keys(casper_types::EntityAddr::SmartContract(
        addressable_cep18_token.value(),
    ));
    assert!(entity_with_named_keys.get("__events").is_none());

    //TODO GR
    //  let entity = entity(&builder, &addressable_cep18_token);
    assert!(builder
        .message_topics(None, cep18_contract_hash.value())
        .unwrap()
        .is_empty());
}

#[test]
fn should_have_native_events() {
    let (
        mut builder,
        TestContext {
            cep18_contract_hash,
            ..
        },
    ) = setup_with_args(runtime_args! {
        ARG_NAME => TOKEN_NAME,
        ARG_SYMBOL => TOKEN_SYMBOL,
        ARG_DECIMALS => TOKEN_DECIMALS,
        ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
        EVENTS_MODE => 2_u8,
        ENABLE_MINT_BURN => true,
    });

    let addressable_cep18_token = AddressableEntityHash::new(cep18_contract_hash.value());
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
    // TODO GR
    // let entity = entity(&builder, &cep18_token);

    let binding = builder
        .message_topics(None, cep18_contract_hash.value())
        .unwrap();
    let (topic_name, message_topic_hash) = binding
        .iter()
        .last()
        .expect("should have at least one topic");

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
    let mint_message = "{\"recipient\":\"entity-account-2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a\",\"amount\":\"1000000\"}";
    let message = Message::new(
        cep18_token.value(),
        mint_message.into(),
        EVENTS.to_string(),
        *message_topic_hash,
        2,
        2,
    );
    assert_eq!(messages, &vec![message]);
}

#[test]
fn should_have_ces_events() {
    let (
        mut builder,
        TestContext {
            cep18_contract_hash,
            ..
        },
    ) = setup_with_args(runtime_args! {
        ARG_NAME => TOKEN_NAME,
        ARG_SYMBOL => TOKEN_SYMBOL,
        ARG_DECIMALS => TOKEN_DECIMALS,
        ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
        EVENTS_MODE => 1_u8,
        ENABLE_MINT_BURN => true,
    });

    let addressable_cep18_token = AddressableEntityHash::new(cep18_contract_hash.value());
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

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        METHOD_MINT,
        runtime_args! {OWNER => TOKEN_OWNER_ADDRESS_1, AMOUNT => U256::from(TOKEN_OWNER_AMOUNT_1)},
    )
    .build();

    builder.exec(mint_request).expect_success().commit();

    let stored_event: Bytes = get_dictionary_value_from_key(
        &builder,
        &Key::addressable_entity_key(EntityKindTag::SmartContract, addressable_cep18_token),
        "__events",
        "1",
    );
    assert_eq!(
        stored_event,
        Bytes::from(vec![
            10, 0, 0, 0, 101, 118, 101, 110, 116, 95, 77, 105, 110, 116, 17, 1, 42, 42, 42, 42, 42,
            42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42,
            42, 42, 42, 42, 42, 3, 64, 66, 15
        ])
    )
}

#[test]
fn should_test_error_message_topic_on_mint_overflow() {
    let (
        mut builder,
        TestContext {
            cep18_contract_hash,
            ..
        },
    ) = setup_with_args(runtime_args! {
        ARG_NAME => TOKEN_NAME,
        ARG_SYMBOL => TOKEN_SYMBOL,
        ARG_DECIMALS => TOKEN_DECIMALS,
        ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
        EVENTS_MODE => 0_u8,
        ENABLE_MINT_BURN => true,
    });

    let addressable_cep18_token = AddressableEntityHash::new(cep18_contract_hash.value());

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        addressable_cep18_token,
        METHOD_MINT,
        runtime_args! {OWNER => TOKEN_OWNER_ADDRESS_1, AMOUNT => U256::MAX},
    )
    .build();

    builder.exec(mint_request).expect_failure().commit();

    let _ = builder.get_exec_result_owned(2).unwrap();
}
