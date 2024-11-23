/* COWL */
use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    PRODUCTION_RUN_GENESIS_REQUEST,
};
use casper_types::{runtime_args, ContractHash, Key, RuntimeArgs, U256};
use cep18_test_contract::constants::{
    ARG_FILTER_CONTRACT_RETURN_VALUE, ARG_TOKEN_CONTRACT, CEP18_TEST_CONTRACT_NAME,
    ENTRY_POINT_SET_FILTER_CONTRACT_RETURN_VALUE, ENTRY_POINT_TRANSFER_FILTER_METHOD,
};
use cowl_cep18::{
    constants::{
        ARG_AMOUNT, ARG_DECIMALS, ARG_NAME, ARG_OWNER, ARG_RECIPIENT, ARG_SYMBOL, ARG_TOTAL_SUPPLY,
        ARG_TRANSFER_FILTER_CONTRACT, ARG_TRANSFER_FILTER_METHOD, ENTRY_POINT_INIT,
        ENTRY_POINT_TRANSFER, ENTRY_POINT_TRANSFER_FROM,
    },
    error::Cep18Error,
    modalities::TransferFilterContractResult,
};

use crate::utility::{
    constants::{
        ACCOUNT_USER_1, ACCOUNT_USER_2, CEP18_CONTRACT_WASM, CEP18_TEST_CONTRACT_WASM,
        CEP18_TEST_TOKEN_CONTRACT_NAME, TOKEN_DECIMALS, TOKEN_NAME, TOKEN_SYMBOL,
        TOKEN_TOTAL_SUPPLY,
    },
    installer_request_builders::{cep18_check_balance_of, make_cep18_approve_request},
    support::{assert_expected_error, create_funded_dummy_account},
};

#[test]
fn check_transfers_with_transfer_filter_contract() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST);

    let account_user_1 = create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1));
    let account_user_2 = create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_2));

    // Install filter contract first with empty TOKEN_CONTRACT value, we will update it after token
    // installation
    let install_request_contract_test = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CEP18_TEST_CONTRACT_WASM,
        runtime_args! {
            ARG_TOKEN_CONTRACT => Key::from(ContractHash::from([0u8; 32])),
        },
    )
    .build();

    builder
        .exec(install_request_contract_test)
        .expect_success()
        .commit();

    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let transfer_filter_contract = account
        .named_keys()
        .get(CEP18_TEST_CONTRACT_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have contract hash");

    let install_args = runtime_args! {
        ARG_NAME => TOKEN_NAME,
        ARG_TRANSFER_FILTER_CONTRACT=> Key::from(transfer_filter_contract),
        ARG_TRANSFER_FILTER_METHOD => ENTRY_POINT_TRANSFER_FILTER_METHOD,
        ARG_SYMBOL => TOKEN_SYMBOL,
        ARG_DECIMALS => TOKEN_DECIMALS,
        ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
    };

    // Install token
    let install_request_contract =
        ExecuteRequestBuilder::standard(*DEFAULT_ACCOUNT_ADDR, CEP18_CONTRACT_WASM, install_args)
            .build();

    builder
        .exec(install_request_contract)
        .expect_success()
        .commit();

    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let cep18_token = account
        .named_keys()
        .get(CEP18_TEST_TOKEN_CONTRACT_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have contract hash");

    let transfer_filter_contract_stored: ContractHash = builder
        .get_value::<Option<ContractHash>>(cep18_token, ARG_TRANSFER_FILTER_CONTRACT)
        .unwrap();
    let transfer_filter_method_stored: String = builder
        .get_value::<Option<String>>(cep18_token, ARG_TRANSFER_FILTER_METHOD)
        .unwrap();

    assert_eq!(transfer_filter_contract_stored, transfer_filter_contract);
    assert_eq!(
        transfer_filter_method_stored,
        ENTRY_POINT_TRANSFER_FILTER_METHOD
    );

    // Update test contract TOKEN_CONTRACT value
    let set_token_contract_request_for_transfer_filter_contract =
        ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            transfer_filter_contract,
            ENTRY_POINT_INIT,
            runtime_args! {
                ARG_TOKEN_CONTRACT => Key::from(cep18_token)
            },
        )
        .build();

    builder
        .exec(set_token_contract_request_for_transfer_filter_contract)
        .expect_success()
        .commit();

    let contract = builder
        .get_contract(transfer_filter_contract)
        .expect("should have contract");
    let named_keys = contract.named_keys();
    let token_contract_stored = *named_keys.get(ARG_TOKEN_CONTRACT).unwrap();

    assert_eq!(token_contract_stored, Key::from(cep18_token));

    let recipient_user_1 = Key::from(account_user_1);
    let amount: U256 = U256::one();

    let cep18_transfer_args = runtime_args! {
        ARG_RECIPIENT => recipient_user_1,
        ARG_AMOUNT => amount,
    };

    let failing_transfer_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        ENTRY_POINT_TRANSFER,
        cep18_transfer_args.clone(),
    )
    .build();

    builder.exec(failing_transfer_call).expect_failure();

    let error = builder.get_error().expect("must have error");

    assert_expected_error(
        error,
        Cep18Error::TransferFilterContractDenied as u16,
        "should not allow transfer with default TransferFilterContractResult::DenyTransfer",
    );

    let actual_balance_to = cep18_check_balance_of(&mut builder, &cep18_token, recipient_user_1);

    assert_eq!(actual_balance_to, U256::zero());

    let actual_balance_from = cep18_check_balance_of(
        &mut builder,
        &cep18_token,
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
    );
    let expected_balance_from = U256::from(TOKEN_TOTAL_SUPPLY);

    assert_eq!(actual_balance_from, expected_balance_from);

    // Simulate filter to accept transfers
    let transfer_filter_contract_set_return_value_request =
        ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            transfer_filter_contract,
            ENTRY_POINT_SET_FILTER_CONTRACT_RETURN_VALUE,
            runtime_args! {
                ARG_FILTER_CONTRACT_RETURN_VALUE => TransferFilterContractResult::ProceedTransfer
            },
        )
        .build();

    builder
        .exec(transfer_filter_contract_set_return_value_request)
        .expect_success()
        .commit();

    let transfer_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        ENTRY_POINT_TRANSFER,
        cep18_transfer_args,
    )
    .build();

    builder.exec(transfer_call).expect_success().commit();

    let actual_balance_to = cep18_check_balance_of(&mut builder, &cep18_token, recipient_user_1);

    assert_eq!(actual_balance_to, amount);

    let actual_balance_from = cep18_check_balance_of(
        &mut builder,
        &cep18_token,
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
    );
    let expected_balance_from = U256::from(TOKEN_TOTAL_SUPPLY) - 1;

    assert_eq!(actual_balance_from, expected_balance_from);

    // Reset filter return
    let transfer_filter_contract_set_return_value_request =
        ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            transfer_filter_contract,
            ENTRY_POINT_SET_FILTER_CONTRACT_RETURN_VALUE,
            runtime_args! {
                ARG_FILTER_CONTRACT_RETURN_VALUE => TransferFilterContractResult::DenyTransfer
            },
        )
        .build();

    builder
        .exec(transfer_filter_contract_set_return_value_request)
        .expect_success()
        .commit();

    let recipient_user_2 = Key::from(account_user_2);

    let cep18_transfer_args = runtime_args! {
        ARG_OWNER => recipient_user_1,
        ARG_RECIPIENT => recipient_user_2,
        ARG_AMOUNT => amount,
    };

    let failing_transfer_call = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_2,
        cep18_token,
        ENTRY_POINT_TRANSFER_FROM,
        cep18_transfer_args.clone(),
    )
    .build();

    builder.exec(failing_transfer_call).expect_failure();

    let approve_request =
        make_cep18_approve_request(recipient_user_1, &cep18_token, recipient_user_2, amount);

    builder.exec(approve_request).expect_success().commit();

    let failing_transfer_call = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_2,
        cep18_token,
        ENTRY_POINT_TRANSFER_FROM,
        cep18_transfer_args.clone(),
    )
    .build();

    builder.exec(failing_transfer_call).expect_failure();

    // Simulate filter to accept transfers
    let transfer_filter_contract_set_return_value_request =
        ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            transfer_filter_contract,
            ENTRY_POINT_SET_FILTER_CONTRACT_RETURN_VALUE,
            runtime_args! {
                ARG_FILTER_CONTRACT_RETURN_VALUE => TransferFilterContractResult::ProceedTransfer
            },
        )
        .build();

    builder
        .exec(transfer_filter_contract_set_return_value_request)
        .expect_success()
        .commit();

    let failing_transfer_call = ExecuteRequestBuilder::contract_call_by_hash(
        account_user_2,
        cep18_token,
        ENTRY_POINT_TRANSFER_FROM,
        cep18_transfer_args.clone(),
    )
    .build();

    builder
        .exec(failing_transfer_call)
        .expect_success()
        .commit();
}

#[test]
fn should_revert_with_invalid_filter_contract_method() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST);

    let install_request_contract_test = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CEP18_TEST_CONTRACT_WASM,
        runtime_args! {
            ARG_TOKEN_CONTRACT => Key::from(ContractHash::from([0u8; 32])),
        },
    )
    .build();

    builder
        .exec(install_request_contract_test)
        .expect_success()
        .commit();

    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let transfer_filter_contract = account
        .named_keys()
        .get(CEP18_TEST_CONTRACT_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have contract hash");

    let install_args = runtime_args! {
        ARG_NAME => TOKEN_NAME,
        ARG_TRANSFER_FILTER_CONTRACT => Key::from(transfer_filter_contract),
        ARG_SYMBOL => TOKEN_SYMBOL,
        ARG_DECIMALS => TOKEN_DECIMALS,
        ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
    };

    // Install token
    let install_request_contract =
        ExecuteRequestBuilder::standard(*DEFAULT_ACCOUNT_ADDR, CEP18_CONTRACT_WASM, install_args)
            .build();

    builder.exec(install_request_contract).expect_failure();

    let error = builder.get_error().expect("must have error");

    assert_expected_error(
        error,
        // Cep18Error::InvalidTransferFilterMethod as u16,
        50002,
        "should not allow installation with filter contract withtout filter contract method",
    );

    let install_args = runtime_args! {
        ARG_NAME => TOKEN_NAME,
        ARG_TRANSFER_FILTER_CONTRACT => Key::from(transfer_filter_contract),
        ARG_SYMBOL => TOKEN_SYMBOL,
        ARG_DECIMALS => TOKEN_DECIMALS,
        ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
        ARG_TRANSFER_FILTER_METHOD => "", // test empty method
    };

    // Install token
    let install_request_contract =
        ExecuteRequestBuilder::standard(*DEFAULT_ACCOUNT_ADDR, CEP18_CONTRACT_WASM, install_args)
            .build();

    builder.exec(install_request_contract).expect_failure();

    let error = builder.get_error().expect("must have error");

    assert_expected_error(
        error,
        // Cep18Error::InvalidTransferFilterMethod as u16,
        50002,
        "should not allow installation with filter contract and empty filter contract method",
    );
}
