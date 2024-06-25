use casper_engine_test_support::{ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR};
use casper_types::{
    addressable_entity::EntityKindTag, runtime_args, AddressableEntityHash, ApiError, EntityAddr,
    Key, U256,
};

use crate::utility::{
    constants::{
        ACCOUNT_1_ADDR, ALLOWANCE_AMOUNT_1, ALLOWANCE_AMOUNT_2, ARG_AMOUNT, ARG_OWNER,
        ARG_RECIPIENT, ARG_SPENDER, DECREASE_ALLOWANCE, ERROR_INSUFFICIENT_ALLOWANCE,
        INCREASE_ALLOWANCE, METHOD_APPROVE, METHOD_TRANSFER_FROM,
    },
    installer_request_builders::{
        cep18_check_allowance_of, make_cep18_approve_request, setup, test_approve_for, TestContext,
    },
};
use casper_execution_engine::{engine_state::Error as CoreError, execution::ExecError};

#[test]
fn should_approve_funds_contract_to_account() {
    let (mut builder, test_context) = setup();
    let TestContext {
        cep18_test_contract_package,
        ..
    } = test_context;

    test_approve_for(
        &mut builder,
        &test_context,
        Key::addressable_entity_key(
            EntityKindTag::SmartContract,
            AddressableEntityHash::new(cep18_test_contract_package.value()),
        ),
        Key::addressable_entity_key(
            EntityKindTag::SmartContract,
            AddressableEntityHash::new(cep18_test_contract_package.value()),
        ),
        Key::AddressableEntity(EntityAddr::Account(DEFAULT_ACCOUNT_ADDR.value())),
    );
}

#[test]
fn should_approve_funds_contract_to_contract() {
    let (mut builder, test_context) = setup();
    let TestContext {
        cep18_test_contract_package,
        ..
    } = test_context;

    test_approve_for(
        &mut builder,
        &test_context,
        Key::AddressableEntity(EntityAddr::SmartContract(
            cep18_test_contract_package.value(),
        )),
        Key::AddressableEntity(EntityAddr::SmartContract(
            cep18_test_contract_package.value(),
        )),
        Key::AddressableEntity(EntityAddr::SmartContract([42; 32])),
    );
}

#[test]
fn should_approve_funds_account_to_account() {
    let (mut builder, test_context) = setup();

    test_approve_for(
        &mut builder,
        &test_context,
        Key::AddressableEntity(EntityAddr::Account(DEFAULT_ACCOUNT_ADDR.value())),
        Key::AddressableEntity(EntityAddr::Account(DEFAULT_ACCOUNT_ADDR.value())),
        Key::AddressableEntity(EntityAddr::Account(ACCOUNT_1_ADDR.value())),
    );
}

#[test]
fn should_approve_funds_account_to_contract() {
    let (mut builder, test_context) = setup();
    test_approve_for(
        &mut builder,
        &test_context,
        Key::AddressableEntity(EntityAddr::Account(DEFAULT_ACCOUNT_ADDR.value())),
        Key::AddressableEntity(EntityAddr::Account(DEFAULT_ACCOUNT_ADDR.value())),
        Key::AddressableEntity(EntityAddr::SmartContract([42; 32])),
    );
}

#[test]
fn should_not_transfer_from_without_enough_allowance() {
    let (
        mut builder,
        TestContext {
            cep18_contract_hash,
            ..
        },
    ) = setup();

    let addressable_cep18_contract_hash = AddressableEntityHash::new(cep18_contract_hash.value());

    let allowance_amount_1 = U256::from(ALLOWANCE_AMOUNT_1);
    let transfer_from_amount_1 = allowance_amount_1 + U256::one();

    let sender = *DEFAULT_ACCOUNT_ADDR;
    let owner = sender;
    let recipient = *ACCOUNT_1_ADDR;

    let cep18_approve_args = runtime_args! {
        ARG_OWNER => Key::AddressableEntity(EntityAddr::Account(owner.value())),
        ARG_SPENDER => Key::AddressableEntity(EntityAddr::Account(recipient.value())),
        ARG_AMOUNT => allowance_amount_1,
    };
    let cep18_transfer_from_args = runtime_args! {
        ARG_OWNER => Key::AddressableEntity(EntityAddr::Account(owner.value())),
        ARG_RECIPIENT => Key::AddressableEntity(EntityAddr::Account(recipient.value())),
        ARG_AMOUNT => transfer_from_amount_1,
    };

    let spender_allowance_before = cep18_check_allowance_of(
        &mut builder,
        Key::AddressableEntity(EntityAddr::Account(owner.value())),
        Key::AddressableEntity(EntityAddr::Account(recipient.value())),
    );
    assert_eq!(spender_allowance_before, U256::zero());

    let approve_request_1 = ExecuteRequestBuilder::contract_call_by_hash(
        sender,
        addressable_cep18_contract_hash,
        METHOD_APPROVE,
        cep18_approve_args,
    )
    .build();

    let transfer_from_request_1 = ExecuteRequestBuilder::contract_call_by_hash(
        sender,
        addressable_cep18_contract_hash,
        METHOD_TRANSFER_FROM,
        cep18_transfer_from_args,
    )
    .build();

    builder.exec(approve_request_1).expect_success().commit();

    let account_1_allowance_after = cep18_check_allowance_of(
        &mut builder,
        Key::AddressableEntity(EntityAddr::Account(owner.value())),
        Key::AddressableEntity(EntityAddr::Account(recipient.value())),
    );
    assert_eq!(account_1_allowance_after, allowance_amount_1);

    builder.exec(transfer_from_request_1).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == ERROR_INSUFFICIENT_ALLOWANCE),
        "{:?}",
        error
    );
}

#[test]
fn test_decrease_allowance() {
    let (
        mut builder,
        TestContext {
            cep18_contract_hash,
            ..
        },
    ) = setup();

    let addressable_cep18_contract_hash = AddressableEntityHash::new(cep18_contract_hash.value());

    let owner = *DEFAULT_ACCOUNT_ADDR;
    let spender = Key::Hash([42; 32]);

    let owner_key = Key::AddressableEntity(EntityAddr::Account(owner.value()));
    let spender_key = Key::AddressableEntity(EntityAddr::SmartContract([42; 32]));

    let allowance_amount_1 = U256::from(ALLOWANCE_AMOUNT_1);
    let allowance_amount_2 = U256::from(ALLOWANCE_AMOUNT_2);

    let spender_allowance_before = cep18_check_allowance_of(&mut builder, owner_key, spender_key);
    assert_eq!(spender_allowance_before, U256::zero());

    let approve_request =
        make_cep18_approve_request(owner_key, &cep18_contract_hash, spender, allowance_amount_1);
    let decrease_allowance_request = ExecuteRequestBuilder::contract_call_by_hash(
        owner,
        addressable_cep18_contract_hash,
        DECREASE_ALLOWANCE,
        runtime_args! {
            ARG_SPENDER => spender,
            ARG_AMOUNT => allowance_amount_2,
        },
    )
    .build();
    let increase_allowance_request = ExecuteRequestBuilder::contract_call_by_hash(
        owner,
        addressable_cep18_contract_hash,
        INCREASE_ALLOWANCE,
        runtime_args! {
            ARG_SPENDER => spender,
            ARG_AMOUNT => allowance_amount_1,
        },
    )
    .build();

    builder.exec(approve_request).expect_success().commit();

    let account_1_allowance_after = cep18_check_allowance_of(&mut builder, owner_key, spender);

    assert_eq!(account_1_allowance_after, allowance_amount_1);

    builder
        .exec(decrease_allowance_request)
        .expect_success()
        .commit();

    let account_1_allowance_after_decrease =
        cep18_check_allowance_of(&mut builder, owner_key, spender);

    assert_eq!(
        account_1_allowance_after_decrease,
        allowance_amount_1 - allowance_amount_2
    );

    builder
        .exec(increase_allowance_request)
        .expect_success()
        .commit();

    let account_1_allowance_after_increase =
        cep18_check_allowance_of(&mut builder, owner_key, spender);

    assert_eq!(
        account_1_allowance_after_increase,
        (allowance_amount_1 * 2) - allowance_amount_2
    );
}
