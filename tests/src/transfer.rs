use casper_engine_test_support::{ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR};
use casper_types::{
    addressable_entity::EntityKindTag, runtime_args, AddressableEntityHash, ApiError, EntityAddr,
    Key, U256,
};

use crate::utility::{
    constants::{
        ACCOUNT_1_ADDR, ACCOUNT_2_ADDR, ALLOWANCE_AMOUNT_1, ARG_AMOUNT, ARG_OWNER, ARG_RECIPIENT,
        ARG_SPENDER, ARG_TOKEN_CONTRACT, ERROR_INSUFFICIENT_BALANCE, METHOD_APPROVE,
        METHOD_FROM_AS_STORED_CONTRACT, METHOD_TRANSFER, METHOD_TRANSFER_FROM, TOKEN_TOTAL_SUPPLY,
        TOTAL_SUPPLY_KEY, TRANSFER_AMOUNT_1,
    },
    installer_request_builders::{
        cep18_check_allowance_of, cep18_check_balance_of, make_cep18_approve_request,
        make_cep18_transfer_request, setup, test_cep18_transfer, TestContext,
    },
};

use casper_execution_engine::{engine_state::Error as CoreError, execution::ExecError};

#[test]
fn should_transfer_full_owned_amount() {
    let (mut builder, TestContext { cep18_token, .. }) = setup();

    let addressable_cep18_token = AddressableEntityHash::new(cep18_token.value());
    let initial_supply = U256::from(TOKEN_TOTAL_SUPPLY);
    let transfer_amount_1 = initial_supply;

    let transfer_1_sender = *DEFAULT_ACCOUNT_ADDR;
    let cep18_transfer_1_args = runtime_args! {
        ARG_RECIPIENT => Key::AddressableEntity(EntityAddr::Account(ACCOUNT_1_ADDR.value())),
        ARG_AMOUNT => transfer_amount_1,
    };

    let owner_balance_before = cep18_check_balance_of(
        &mut builder,
        &cep18_token,
        Key::AddressableEntity(EntityAddr::Account(DEFAULT_ACCOUNT_ADDR.value())),
    );
    assert_eq!(owner_balance_before, initial_supply);

    let account_1_balance_before =
        cep18_check_balance_of(&mut builder, &cep18_token, Key::AddressableEntity(EntityAddr::Account(ACCOUNT_1_ADDR.value())));
    assert_eq!(account_1_balance_before, U256::zero());

    let token_transfer_request_1 = ExecuteRequestBuilder::contract_call_by_hash(
        transfer_1_sender,
        addressable_cep18_token,
        METHOD_TRANSFER,
        cep18_transfer_1_args,
    )
    .build();

    builder
        .exec(token_transfer_request_1)
        .expect_success()
        .commit();

    let account_1_balance_after =
        cep18_check_balance_of(&mut builder, &cep18_token, Key::AddressableEntity(EntityAddr::Account(ACCOUNT_1_ADDR.value())));
    assert_eq!(account_1_balance_after, transfer_amount_1);

    let owner_balance_after = cep18_check_balance_of(
        &mut builder,
        &cep18_token,
        Key::AddressableEntity(EntityAddr::Account(DEFAULT_ACCOUNT_ADDR.value())),
    );
    assert_eq!(owner_balance_after, U256::zero());

    let total_supply: U256 = builder.get_value(
        EntityAddr::new_smart_contract(cep18_token.value()),
        TOTAL_SUPPLY_KEY,
    );
    assert_eq!(total_supply, initial_supply);
}

#[test]
fn should_not_transfer_more_than_owned_balance() {
    let (mut builder, TestContext { cep18_token, .. }) = setup();

    let addressable_cep18_token = AddressableEntityHash::new(cep18_token.value());

    let initial_supply = U256::from(TOKEN_TOTAL_SUPPLY);
    let transfer_amount = initial_supply + U256::one();

    let transfer_1_sender = *DEFAULT_ACCOUNT_ADDR;
    let transfer_1_recipient = *ACCOUNT_1_ADDR;

    let cep18_transfer_1_args = runtime_args! {
        ARG_RECIPIENT => Key::AddressableEntity(EntityAddr::Account(transfer_1_recipient.value())),
        ARG_AMOUNT => transfer_amount,
    };

    let owner_balance_before = cep18_check_balance_of(
        &mut builder,
        &cep18_token,
        Key::AddressableEntity(EntityAddr::Account(DEFAULT_ACCOUNT_ADDR.value())),
    );
    assert_eq!(owner_balance_before, initial_supply);
    assert!(transfer_amount > owner_balance_before);

    let account_1_balance_before =
        cep18_check_balance_of(&mut builder, &cep18_token, Key::AddressableEntity(EntityAddr::Account(ACCOUNT_1_ADDR.value())));
    assert_eq!(account_1_balance_before, U256::zero());

    let token_transfer_request_1 = ExecuteRequestBuilder::contract_call_by_hash(
        transfer_1_sender,
        addressable_cep18_token,
        METHOD_TRANSFER,
        cep18_transfer_1_args,
    )
    .build();

    builder.exec(token_transfer_request_1).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == ERROR_INSUFFICIENT_BALANCE),
        "{:?}",
        error
    );

    let account_1_balance_after = cep18_check_balance_of(
        &mut builder,
        &cep18_token,
        Key::AddressableEntity(EntityAddr::Account(transfer_1_recipient.value())),
    );
    assert_eq!(account_1_balance_after, account_1_balance_before);

    let owner_balance_after =
        cep18_check_balance_of(&mut builder, &cep18_token, Key::AddressableEntity(EntityAddr::Account(transfer_1_sender.value())));
    assert_eq!(owner_balance_after, initial_supply);

    let total_supply: U256 = builder.get_value(
        EntityAddr::new_smart_contract(cep18_token.value()),
        TOTAL_SUPPLY_KEY,
    );
    assert_eq!(total_supply, initial_supply);
}

#[test]
fn should_transfer_from_from_account_to_account() {
    let (mut builder, TestContext { cep18_token, .. }) = setup();

    let addressable_cep18_token = AddressableEntityHash::new(cep18_token.value());

    let initial_supply = U256::from(TOKEN_TOTAL_SUPPLY);
    let allowance_amount_1 = U256::from(ALLOWANCE_AMOUNT_1);
    let transfer_from_amount_1 = allowance_amount_1;

    let owner = *DEFAULT_ACCOUNT_ADDR;
    let spender = *ACCOUNT_1_ADDR;

    let cep18_approve_args = runtime_args! {
        ARG_OWNER => Key::AddressableEntity(EntityAddr::Account(owner.value())),
        ARG_SPENDER => Key::AddressableEntity(EntityAddr::Account(spender.value())),
        ARG_AMOUNT => allowance_amount_1,
    };
    let cep18_transfer_from_args = runtime_args! {
        ARG_OWNER => Key::AddressableEntity(EntityAddr::Account(owner.value())),
        ARG_RECIPIENT => Key::AddressableEntity(EntityAddr::Account(spender.value())),
        ARG_AMOUNT => transfer_from_amount_1,
    };

    let spender_allowance_before =
        cep18_check_allowance_of(&mut builder, Key::AddressableEntity(EntityAddr::Account(owner.value())), Key::AddressableEntity(EntityAddr::Account(spender.value())));
    assert_eq!(spender_allowance_before, U256::zero());

    let approve_request_1 = ExecuteRequestBuilder::contract_call_by_hash(
        owner,
        addressable_cep18_token,
        METHOD_APPROVE,
        cep18_approve_args,
    )
    .build();

    let transfer_from_request_1 = ExecuteRequestBuilder::contract_call_by_hash(
        spender,
        addressable_cep18_token,
        METHOD_TRANSFER_FROM,
        cep18_transfer_from_args,
    )
    .build();

    builder.exec(approve_request_1).expect_success().commit();

    let account_1_balance_before =
        cep18_check_balance_of(&mut builder, &cep18_token, Key::AddressableEntity(EntityAddr::Account(owner.value())));
    assert_eq!(account_1_balance_before, initial_supply);

    let account_1_allowance_before =
        cep18_check_allowance_of(&mut builder, Key::AddressableEntity(EntityAddr::Account(owner.value())), Key::AddressableEntity(EntityAddr::Account(spender.value())));
    assert_eq!(account_1_allowance_before, allowance_amount_1);

    builder
        .exec(transfer_from_request_1)
        .expect_success()
        .commit();

    let account_1_allowance_after =
        cep18_check_allowance_of(&mut builder, Key::AddressableEntity(EntityAddr::Account(owner.value())), Key::AddressableEntity(EntityAddr::Account(spender.value())));
    assert_eq!(
        account_1_allowance_after,
        account_1_allowance_before - transfer_from_amount_1
    );

    let account_1_balance_after =
        cep18_check_balance_of(&mut builder, &cep18_token, Key::AddressableEntity(EntityAddr::Account(owner.value())));
    assert_eq!(
        account_1_balance_after,
        account_1_balance_before - transfer_from_amount_1
    );
}

#[test]
fn should_transfer_from_account_by_contract() {
    let (
        mut builder,
        TestContext {
            cep18_token,
            cep18_test_contract_package,
            ..
        },
    ) = setup();

    let addressable_cep18_token = AddressableEntityHash::new(cep18_token.value());

    let initial_supply = U256::from(TOKEN_TOTAL_SUPPLY);
    let allowance_amount_1 = U256::from(ALLOWANCE_AMOUNT_1);
    let transfer_from_amount_1 = allowance_amount_1;

    let owner = *DEFAULT_ACCOUNT_ADDR;

    let spender = Key::AddressableEntity(EntityAddr::SmartContract(cep18_test_contract_package.value()));
    let recipient = Key::AddressableEntity(EntityAddr::Account(ACCOUNT_1_ADDR.value()));

    let cep18_approve_args = runtime_args! {
        ARG_OWNER => Key::AddressableEntity(EntityAddr::Account(owner.value())),
        ARG_SPENDER => spender,
        ARG_AMOUNT => allowance_amount_1,
    };
    let cep18_transfer_from_args = runtime_args! {
        ARG_TOKEN_CONTRACT => Key::addressable_entity_key(EntityKindTag::SmartContract, cep18_token),
        ARG_OWNER => Key::AddressableEntity(EntityAddr::Account(owner.value())),
        ARG_RECIPIENT => recipient,
        ARG_AMOUNT => transfer_from_amount_1,
    };

    let spender_allowance_before =
        cep18_check_allowance_of(&mut builder, Key::AddressableEntity(EntityAddr::Account(owner.value())), spender);
    assert_eq!(spender_allowance_before, U256::zero());

    let approve_request_1 = ExecuteRequestBuilder::contract_call_by_hash(
        owner,
        addressable_cep18_token,
        METHOD_APPROVE,
        cep18_approve_args,
    )
    .build();

    let transfer_from_request_1 = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_test_contract_package,
        None,
        METHOD_FROM_AS_STORED_CONTRACT,
        cep18_transfer_from_args,
    )
    .build();

    builder.exec(approve_request_1).expect_success().commit();

    let owner_balance_before =
        cep18_check_balance_of(&mut builder, &cep18_token, Key::AddressableEntity(EntityAddr::Account(owner.value())));
    assert_eq!(owner_balance_before, initial_supply);

    let spender_allowance_before =
        cep18_check_allowance_of(&mut builder, Key::AddressableEntity(EntityAddr::Account(owner.value())), spender);
    assert_eq!(spender_allowance_before, allowance_amount_1);

    builder
        .exec(transfer_from_request_1)
        .expect_success()
        .commit();

    let spender_allowance_after =
        cep18_check_allowance_of(&mut builder, Key::AddressableEntity(EntityAddr::Account(owner.value())), spender);
    assert_eq!(
        spender_allowance_after,
        spender_allowance_before - transfer_from_amount_1
    );

    let owner_balance_after =
        cep18_check_balance_of(&mut builder, &cep18_token, Key::AddressableEntity(EntityAddr::Account(owner.value())));
    assert_eq!(
        owner_balance_after,
        owner_balance_before - transfer_from_amount_1
    );
}

#[test]
fn should_not_be_able_to_own_transfer() {
    let (mut builder, TestContext { cep18_token, .. }) = setup();

    let sender = Key::AddressableEntity(EntityAddr::Account(DEFAULT_ACCOUNT_ADDR.value()));
    let recipient = Key::AddressableEntity(EntityAddr::Account(DEFAULT_ACCOUNT_ADDR.value()));

    let transfer_amount = U256::from(TRANSFER_AMOUNT_1);

    let sender_balance_before = cep18_check_balance_of(&mut builder, &cep18_token, sender);
    let recipient_balance_before = cep18_check_balance_of(&mut builder, &cep18_token, recipient);

    assert_eq!(sender_balance_before, recipient_balance_before);

    let token_transfer_request_1 =
        make_cep18_transfer_request(sender, &cep18_token, recipient, transfer_amount);

    builder.exec(token_transfer_request_1).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == 60017),
        "{:?}",
        error
    );
}

#[test]
fn should_not_be_able_to_own_transfer_from() {
    let (mut builder, TestContext { cep18_token, .. }) = setup();

    let addressable_cep18_token = AddressableEntityHash::new(cep18_token.value());

    let sender = *DEFAULT_ACCOUNT_ADDR;
    let owner = Key::AddressableEntity(EntityAddr::Account(sender.value()));
    let spender = Key::AddressableEntity(EntityAddr::Account(sender.value()));
    let sender_key = Key::AddressableEntity(EntityAddr::Account(sender.value()));
    let recipient = Key::AddressableEntity(EntityAddr::Account(sender.value()));

    let allowance_amount = U256::from(ALLOWANCE_AMOUNT_1);
    let transfer_amount = U256::from(TRANSFER_AMOUNT_1);

    let approve_request =
        make_cep18_approve_request(sender_key, &cep18_token, spender, allowance_amount);

    builder.exec(approve_request).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == 60017),
        "{:?}",
        error
    );

    let sender_balance_before = cep18_check_balance_of(&mut builder, &cep18_token, sender_key);
    let recipient_balance_before = cep18_check_balance_of(&mut builder, &cep18_token, recipient);

    assert_eq!(sender_balance_before, recipient_balance_before);

    let transfer_from_request = {
        let cep18_transfer_from_args = runtime_args! {
            ARG_OWNER => owner,
            ARG_RECIPIENT => recipient,
            ARG_AMOUNT => transfer_amount,
        };
        ExecuteRequestBuilder::contract_call_by_hash(
            sender,
            addressable_cep18_token,
            METHOD_TRANSFER_FROM,
            cep18_transfer_from_args,
        )
        .build()
    };

    builder.exec(transfer_from_request).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == 60017),
        "{:?}",
        error
    );
}

#[test]
fn should_verify_zero_amount_transfer_is_noop() {
    let (mut builder, TestContext { cep18_token, .. }) = setup();

    let sender = Key::AddressableEntity(EntityAddr::Account(DEFAULT_ACCOUNT_ADDR.value()));
    let recipient = Key::AddressableEntity(EntityAddr::Account(ACCOUNT_1_ADDR.value()));

    let transfer_amount = U256::zero();

    let sender_balance_before = cep18_check_balance_of(&mut builder, &cep18_token, sender);
    let recipient_balance_before = cep18_check_balance_of(&mut builder, &cep18_token, recipient);

    let token_transfer_request_1 =
        make_cep18_transfer_request(sender, &cep18_token, recipient, transfer_amount);

    builder
        .exec(token_transfer_request_1)
        .expect_success()
        .commit();

    let sender_balance_after = cep18_check_balance_of(&mut builder, &cep18_token, sender);
    assert_eq!(sender_balance_before, sender_balance_after);

    let recipient_balance_after = cep18_check_balance_of(&mut builder, &cep18_token, recipient);
    assert_eq!(recipient_balance_before, recipient_balance_after);
}

#[test]
fn should_verify_zero_amount_transfer_from_is_noop() {
    let (mut builder, TestContext { cep18_token, .. }) = setup();

    let addressable_cep18_token = AddressableEntityHash::new(cep18_token.value());

    let owner = *DEFAULT_ACCOUNT_ADDR;
    let owner_key = Key::AddressableEntity(EntityAddr::Account(owner.value()));
    let spender = Key::AddressableEntity(EntityAddr::Account(ACCOUNT_1_ADDR.value()));
    let recipient = Key::AddressableEntity(EntityAddr::Account(ACCOUNT_2_ADDR.value()));

    let allowance_amount = U256::from(1);
    let transfer_amount = U256::zero();

    let approve_request =
        make_cep18_approve_request(owner_key, &cep18_token, spender, allowance_amount);

    builder.exec(approve_request).expect_success().commit();

    let spender_allowance_before = cep18_check_allowance_of(&mut builder, owner_key, spender);

    let owner_balance_before = cep18_check_balance_of(&mut builder, &cep18_token, owner_key);
    let recipient_balance_before = cep18_check_balance_of(&mut builder, &cep18_token, recipient);

    let transfer_from_request = {
        let cep18_transfer_from_args = runtime_args! {
            ARG_OWNER => owner_key,
            ARG_RECIPIENT => recipient,
            ARG_AMOUNT => transfer_amount,
        };
        ExecuteRequestBuilder::contract_call_by_hash(
            owner,
            addressable_cep18_token,
            METHOD_TRANSFER_FROM,
            cep18_transfer_from_args,
        )
        .build()
    };

    builder
        .exec(transfer_from_request)
        .expect_success()
        .commit();

    let owner_balance_after = cep18_check_balance_of(&mut builder, &cep18_token, owner_key);
    assert_eq!(owner_balance_before, owner_balance_after);

    let recipient_balance_after = cep18_check_balance_of(&mut builder, &cep18_token, recipient);
    assert_eq!(recipient_balance_before, recipient_balance_after);

    let spender_allowance_after = cep18_check_allowance_of(&mut builder, owner_key, spender);
    assert_eq!(spender_allowance_after, spender_allowance_before);
}

#[test]
fn should_transfer_contract_to_contract() {
    let (mut builder, test_context) = setup();
    let TestContext {
        cep18_test_contract_package,
        ..
    } = test_context;

    let sender1 = Key::AddressableEntity(EntityAddr::Account(DEFAULT_ACCOUNT_ADDR.value()));
    let recipient1 = Key::AddressableEntity(EntityAddr::SmartContract(cep18_test_contract_package.value()));
    let sender2 = Key::AddressableEntity(EntityAddr::SmartContract(cep18_test_contract_package.value()));
    let recipient2 = Key::AddressableEntity(EntityAddr::SmartContract([42; 32]));

    test_cep18_transfer(
        &mut builder,
        &test_context,
        sender1,
        recipient1,
        sender2,
        recipient2,
    );
}

#[test]
fn should_transfer_contract_to_account() {
    let (mut builder, test_context) = setup();
    let TestContext {
        cep18_test_contract_package,
        ..
    } = test_context;

    let sender1 = Key::AddressableEntity(EntityAddr::Account(DEFAULT_ACCOUNT_ADDR.value()));
    let recipient1 = Key::AddressableEntity(EntityAddr::SmartContract(cep18_test_contract_package.value()));

    let sender2 = Key::AddressableEntity(EntityAddr::SmartContract(cep18_test_contract_package.value()));
    let recipient2 = Key::AddressableEntity(EntityAddr::Account(ACCOUNT_1_ADDR.value()));

    test_cep18_transfer(
        &mut builder,
        &test_context,
        sender1,
        recipient1,
        sender2,
        recipient2,
    );
}

#[test]
fn should_transfer_account_to_contract() {
    let (mut builder, test_context) = setup();

    let sender1 = Key::AddressableEntity(EntityAddr::Account(DEFAULT_ACCOUNT_ADDR.value()));
    let recipient1 = Key::AddressableEntity(EntityAddr::Account(ACCOUNT_1_ADDR.value()));
    let sender2 = Key::AddressableEntity(EntityAddr::Account(ACCOUNT_1_ADDR.value()));
    let recipient2 = Key::AddressableEntity(EntityAddr::SmartContract(test_context.cep18_test_contract_package.value()));

    test_cep18_transfer(
        &mut builder,
        &test_context,
        sender1,
        recipient1,
        sender2,
        recipient2,
    );
}

#[test]
fn should_transfer_account_to_account() {
    let (mut builder, test_context) = setup();
    let sender1 = Key::AddressableEntity(EntityAddr::Account(DEFAULT_ACCOUNT_ADDR.value()));
    let recipient1 = Key::AddressableEntity(EntityAddr::Account(ACCOUNT_1_ADDR.value()));
    let sender2 = Key::AddressableEntity(EntityAddr::Account(ACCOUNT_1_ADDR.value()));
    let recipient2 = Key::AddressableEntity(EntityAddr::Account(ACCOUNT_2_ADDR.value()));

    test_cep18_transfer(
        &mut builder,
        &test_context,
        sender1,
        recipient1,
        sender2,
        recipient2,
    );
}
