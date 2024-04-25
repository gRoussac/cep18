use casper_engine_test_support::{
    utils::create_run_genesis_request, ExecuteRequest, ExecuteRequestBuilder, LmdbWasmTestBuilder,
    DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_PUBLIC_KEY,
    utils::create_run_genesis_request, ExecuteRequest, ExecuteRequestBuilder, LmdbWasmTestBuilder,
    DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_PUBLIC_KEY,
};
use casper_types::{
    account::AccountHash, addressable_entity::EntityKindTag, bytesrepr::FromBytes,
    runtime_args, AddressableEntityHash, CLTyped, EntityAddr,
    GenesisAccount, Key, Motes, PackageHash, RuntimeArgs, U256, U512,
};

use crate::utility::constants::{
    ALLOWANCE_AMOUNT_1, ALLOWANCE_AMOUNT_2, TOTAL_SUPPLY_KEY,
    TRANSFER_AMOUNT_1, TRANSFER_AMOUNT_2,
};

use super::constants::{
    ACCOUNT_1_PUBLIC_KEY, ACCOUNT_2_PUBLIC_KEY, ARG_ADDRESS, ARG_AMOUNT, ARG_DECIMALS, ARG_NAME,
    ARG_OWNER, ARG_RECIPIENT, ARG_SPENDER, ARG_SYMBOL, ARG_TOKEN_CONTRACT, ARG_TOTAL_SUPPLY,
    ACCOUNT_1_PUBLIC_KEY, ACCOUNT_2_PUBLIC_KEY, ARG_ADDRESS, ARG_AMOUNT, ARG_DECIMALS, ARG_NAME,
    ARG_OWNER, ARG_RECIPIENT, ARG_SPENDER, ARG_SYMBOL, ARG_TOKEN_CONTRACT, ARG_TOTAL_SUPPLY,
    CEP18_CONTRACT_WASM, CEP18_TEST_CONTRACT_KEY, CEP18_TEST_CONTRACT_WASM,
    CEP18_TOKEN_CONTRACT_KEY, CHECK_ALLOWANCE_OF_ENTRYPOINT, CHECK_BALANCE_OF_ENTRYPOINT,
    CHECK_TOTAL_SUPPLY_ENTRYPOINT, METHOD_APPROVE, METHOD_APPROVE_AS_STORED_CONTRACT,
    METHOD_TRANSFER, METHOD_TRANSFER_AS_STORED_CONTRACT, RESULT_KEY, TOKEN_DECIMALS, TOKEN_NAME,
    TOKEN_SYMBOL, TOKEN_TOTAL_SUPPLY,
};

/// Converts hash addr of Account into Hash, and Hash into Account
///
/// This is useful for making sure CEP18 library respects different variants of Key when storing
/// balances.
pub(crate) fn invert_cep18_address(address: Key) -> Key {
    match address {
        Key::Account(account_hash) => Key::Hash(account_hash.value()),
        Key::Hash(contract_hash) => Key::Account(AccountHash::new(contract_hash)),
        _ => panic!("Unsupported Key variant"),
    }
}

#[derive(Copy, Clone)]
pub(crate) struct TestContext {
    pub(crate) cep18_token: AddressableEntityHash,
    pub(crate) cep18_test_contract_package: PackageHash,
    pub(crate) cep18_token: AddressableEntityHash,
    pub(crate) cep18_test_contract_package: PackageHash,
}

pub(crate) fn setup() -> (LmdbWasmTestBuilder, TestContext) {
pub(crate) fn setup() -> (LmdbWasmTestBuilder, TestContext) {
    setup_with_args(runtime_args! {
        ARG_NAME => TOKEN_NAME,
        ARG_SYMBOL => TOKEN_SYMBOL,
        ARG_DECIMALS => TOKEN_DECIMALS,
        ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
    })
}

pub(crate) fn setup_with_args(install_args: RuntimeArgs) -> (LmdbWasmTestBuilder, TestContext) {
    let mut builder = LmdbWasmTestBuilder::default();
    builder.run_genesis(create_run_genesis_request(vec![
        GenesisAccount::Account {
            public_key: DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
            balance: Motes::new(U512::from(5_000_000_000_000_u64)),
            validator: None,
        },
        GenesisAccount::Account {
            public_key: ACCOUNT_1_PUBLIC_KEY.clone(),
            balance: Motes::new(U512::from(5_000_000_000_000_u64)),
            validator: None,
        },
        GenesisAccount::Account {
            public_key: ACCOUNT_2_PUBLIC_KEY.clone(),
            balance: Motes::new(U512::from(5_000_000_000_000_u64)),
            validator: None,
        },
    ]));
pub(crate) fn setup_with_args(install_args: RuntimeArgs) -> (LmdbWasmTestBuilder, TestContext) {
    let mut builder = LmdbWasmTestBuilder::default();
    builder.run_genesis(create_run_genesis_request(vec![
        GenesisAccount::Account {
            public_key: DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
            balance: Motes::new(U512::from(5_000_000_000_000_u64)),
            validator: None,
        },
        GenesisAccount::Account {
            public_key: ACCOUNT_1_PUBLIC_KEY.clone(),
            balance: Motes::new(U512::from(5_000_000_000_000_u64)),
            validator: None,
        },
        GenesisAccount::Account {
            public_key: ACCOUNT_2_PUBLIC_KEY.clone(),
            balance: Motes::new(U512::from(5_000_000_000_000_u64)),
            validator: None,
        },
    ]));

    let install_request_1 =
        ExecuteRequestBuilder::standard(*DEFAULT_ACCOUNT_ADDR, CEP18_CONTRACT_WASM, install_args)
            .build();

    let install_request_2 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CEP18_TEST_CONTRACT_WASM,
        RuntimeArgs::default(),
    )
    .build();

    builder.exec(install_request_1).expect_success().commit();
    builder.exec(install_request_2).expect_success().commit();

    let account =
        builder.get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR).unwrap();
    let account_named_keys = account.named_keys();
    // println!("{account_named_keys:?}");

    // let account_named_keys_2 = builder.get_named_keys(EntityAddr::new_account(DEFAULT_ACCOUNT_ADDR.value()));
    // println!("{account_named_keys_2:?}");

    let cep18_token = account_named_keys
    let cep18_token = account_named_keys
        .get(CEP18_TOKEN_CONTRACT_KEY)
        .and_then(|key| key.into_entity_hash())
        .and_then(|key| key.into_entity_hash())
        .expect("should have contract hash");

    let cep18_test_contract_package = account_named_keys
    let cep18_test_contract_package = account_named_keys
        .get(CEP18_TEST_CONTRACT_KEY)
        .and_then(|key| key.into_package_hash())
        .expect("should have package hash");
        .and_then(|key| key.into_package_hash())
        .expect("should have package hash");

    let test_context = TestContext {
        cep18_token,
        cep18_test_contract_package,
    };

    (builder, test_context)
}

pub(crate) fn cep18_check_total_supply(
    builder: &mut LmdbWasmTestBuilder,
    cep18_contract_hash: &AddressableEntityHash,
    builder: &mut LmdbWasmTestBuilder,
    cep18_contract_hash: &AddressableEntityHash,
) -> U256 {
    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let cep18_test_contract_package = account
        .named_keys()
        .get(CEP18_TEST_CONTRACT_KEY)
        .and_then(|key| key.into_package_hash())
        .and_then(|key| key.into_package_hash())
        .expect("should have test contract hash");

    let check_total_supply_args = runtime_args! {
        ARG_TOKEN_CONTRACT => Key::addressable_entity_key(EntityKindTag::SmartContract, *cep18_contract_hash),
        ARG_TOKEN_CONTRACT => Key::addressable_entity_key(EntityKindTag::SmartContract, *cep18_contract_hash),
    };

    let exec_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_test_contract_package,
        None,
        CHECK_TOTAL_SUPPLY_ENTRYPOINT,
        check_total_supply_args,
    )
    .build();
    builder.exec(exec_request).expect_success().commit();

    get_test_result(builder, cep18_test_contract_package)
}

pub(crate) fn get_test_result<T: FromBytes + CLTyped>(
    builder: &mut LmdbWasmTestBuilder,
    cep18_test_contract_package: PackageHash,
    builder: &mut LmdbWasmTestBuilder,
    cep18_test_contract_package: PackageHash,
) -> T {
    let contract_package = builder
        .get_package(cep18_test_contract_package)
        .get_package(cep18_test_contract_package)
        .expect("should have contract package");
    let enabled_versions = contract_package.enabled_versions();
    let contract_hash = enabled_versions
        .contract_hashes()
        .last()
    let contract_hash = enabled_versions
        .contract_hashes()
        .last()
        .expect("should have latest version");
    let entity_addr = EntityAddr::new_smart_contract(contract_hash.value());
    builder.get_value(entity_addr, RESULT_KEY)
    let entity_addr = EntityAddr::new_smart_contract(contract_hash.value());
    builder.get_value(entity_addr, RESULT_KEY)
}

pub(crate) fn cep18_check_balance_of(
    builder: &mut LmdbWasmTestBuilder,
    cep18_contract_hash: &AddressableEntityHash,
    builder: &mut LmdbWasmTestBuilder,
    cep18_contract_hash: &AddressableEntityHash,
    address: Key,
) -> U256 {
    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let cep18_test_contract_package = account
        .named_keys()
        .get(CEP18_TEST_CONTRACT_KEY)
        .and_then(|key| key.into_package_hash())
        .and_then(|key| key.into_package_hash())
        .expect("should have test contract hash");

    let check_balance_args = runtime_args! {
        ARG_TOKEN_CONTRACT => Key::addressable_entity_key(EntityKindTag::SmartContract, *cep18_contract_hash),
        ARG_TOKEN_CONTRACT => Key::addressable_entity_key(EntityKindTag::SmartContract, *cep18_contract_hash),
        ARG_ADDRESS => address,
    };
    let exec_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_test_contract_package,
        None,
        CHECK_BALANCE_OF_ENTRYPOINT,
        check_balance_args,
    )
    .build();
    builder.exec(exec_request).expect_success().commit();

    get_test_result(builder, cep18_test_contract_package)
}

pub(crate) fn cep18_check_allowance_of(
    builder: &mut LmdbWasmTestBuilder,
    builder: &mut LmdbWasmTestBuilder,
    owner: Key,
    spender: Key,
) -> U256 {
    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");
    let cep18_contract_hash = account
        .named_keys()
        .get(CEP18_TOKEN_CONTRACT_KEY)
        .and_then(|key| key.into_entity_hash())
        .and_then(|key| key.into_entity_hash())
        .expect("should have test contract hash");
    let cep18_test_contract_package = account
        .named_keys()
        .get(CEP18_TEST_CONTRACT_KEY)
        .and_then(|key| key.into_package_hash())
        .and_then(|key| key.into_package_hash())
        .expect("should have test contract hash");

    let check_balance_args = runtime_args! {
        ARG_TOKEN_CONTRACT => Key::addressable_entity_key(EntityKindTag::SmartContract, cep18_contract_hash),
        ARG_TOKEN_CONTRACT => Key::addressable_entity_key(EntityKindTag::SmartContract, cep18_contract_hash),
        ARG_OWNER => owner,
        ARG_SPENDER => spender,
    };
    let exec_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_test_contract_package,
        None,
        CHECK_ALLOWANCE_OF_ENTRYPOINT,
        check_balance_args,
    )
    .build();
    builder.exec(exec_request).expect_success().commit();

    get_test_result(builder, cep18_test_contract_package)
}

pub(crate) fn test_cep18_transfer(
    builder: &mut LmdbWasmTestBuilder,
    builder: &mut LmdbWasmTestBuilder,
    test_context: &TestContext,
    sender1: Key,
    recipient1: Key,
    sender2: Key,
    recipient2: Key,
) {
    let TestContext { cep18_token, .. } = test_context;

    let transfer_amount_1 = U256::from(TRANSFER_AMOUNT_1);
    let transfer_amount_2 = U256::from(TRANSFER_AMOUNT_2);

    let sender_balance_before = cep18_check_balance_of(builder, cep18_token, sender1);
    assert_ne!(sender_balance_before, U256::zero());

    let account_1_balance_before = cep18_check_balance_of(builder, cep18_token, recipient1);
    assert_eq!(account_1_balance_before, U256::zero());

    let account_2_balance_before = cep18_check_balance_of(builder, cep18_token, recipient1);
    assert_eq!(account_2_balance_before, U256::zero());

    let token_transfer_request_1 =
        make_cep18_transfer_request(sender1, cep18_token, recipient1, transfer_amount_1);

    builder
        .exec(token_transfer_request_1)
        .expect_success()
        .commit();

    let account_1_balance_after = cep18_check_balance_of(builder, cep18_token, recipient1);
    assert_eq!(account_1_balance_after, transfer_amount_1);
    let account_1_balance_before = account_1_balance_after;

    let sender_balance_after = cep18_check_balance_of(builder, cep18_token, sender1);
    assert_eq!(
        sender_balance_after,
        sender_balance_before - transfer_amount_1
    );
    let sender_balance_before = sender_balance_after;

    let token_transfer_request_2 =
        make_cep18_transfer_request(sender2, cep18_token, recipient2, transfer_amount_2);

    builder
        .exec(token_transfer_request_2)
        .expect_success()
        .commit();

    let sender_balance_after = cep18_check_balance_of(builder, cep18_token, sender1);
    assert_eq!(sender_balance_after, sender_balance_before);

    let account_1_balance_after = cep18_check_balance_of(builder, cep18_token, recipient1);
    assert!(account_1_balance_after < account_1_balance_before);
    assert_eq!(
        account_1_balance_after,
        transfer_amount_1 - transfer_amount_2
    );

    let account_2_balance_after = cep18_check_balance_of(builder, cep18_token, recipient2);
    assert_eq!(account_2_balance_after, transfer_amount_2);
}

pub(crate) fn make_cep18_transfer_request(
    sender: Key,
    cep18_token: &AddressableEntityHash,
    cep18_token: &AddressableEntityHash,
    recipient: Key,
    amount: U256,
) -> ExecuteRequest {
    match sender {
        Key::Account(sender) => ExecuteRequestBuilder::contract_call_by_hash(
            sender,
            AddressableEntityHash::new(cep18_token.value()),
            AddressableEntityHash::new(cep18_token.value()),
            METHOD_TRANSFER,
            runtime_args! {
                ARG_AMOUNT => amount,
                ARG_RECIPIENT => recipient,
            },
        )
        .build(),
        Key::Hash(contract_package_hash) => ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            PackageHash::new(contract_package_hash),
            PackageHash::new(contract_package_hash),
            None,
            METHOD_TRANSFER_AS_STORED_CONTRACT,
            runtime_args! {
                ARG_TOKEN_CONTRACT => Key::addressable_entity_key(EntityKindTag::SmartContract, *cep18_token),
                ARG_TOKEN_CONTRACT => Key::addressable_entity_key(EntityKindTag::SmartContract, *cep18_token),
                ARG_AMOUNT => amount,
                ARG_RECIPIENT => recipient,
            },
        )
        .build(),
        _ => panic!("Unknown variant"),
    }
}

pub(crate) fn make_cep18_approve_request(
    sender: Key,
    cep18_token: &AddressableEntityHash,
    cep18_token: &AddressableEntityHash,
    spender: Key,
    amount: U256,
) -> ExecuteRequest {
    match sender {
        Key::Account(sender) => ExecuteRequestBuilder::contract_call_by_hash(
            sender,
            AddressableEntityHash::new(cep18_token.value()),
            AddressableEntityHash::new(cep18_token.value()),
            METHOD_APPROVE,
            runtime_args! {
                ARG_SPENDER => spender,
                ARG_AMOUNT => amount,
            },
        )
        .build(),
        Key::Hash(contract_package_hash) => ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            PackageHash::new(contract_package_hash),
            PackageHash::new(contract_package_hash),
            None,
            METHOD_APPROVE_AS_STORED_CONTRACT,
            runtime_args! {
                ARG_TOKEN_CONTRACT => Key::addressable_entity_key(EntityKindTag::SmartContract, *cep18_token),
                ARG_TOKEN_CONTRACT => Key::addressable_entity_key(EntityKindTag::SmartContract, *cep18_token),
                ARG_SPENDER => spender,
                ARG_AMOUNT => amount,
            },
        )
        .build(),
        _ => panic!("Unknown variant"),
    }
}

pub(crate) fn test_approve_for(
    builder: &mut LmdbWasmTestBuilder,
    builder: &mut LmdbWasmTestBuilder,
    test_context: &TestContext,
    sender: Key,
    owner: Key,
    spender: Key,
) {
    let TestContext { cep18_token, .. } = test_context;
    let initial_supply = U256::from(TOKEN_TOTAL_SUPPLY);
    let allowance_amount_1 = U256::from(ALLOWANCE_AMOUNT_1);
    let allowance_amount_2 = U256::from(ALLOWANCE_AMOUNT_2);

    let spender_allowance_before = cep18_check_allowance_of(builder, owner, spender);
    assert_eq!(spender_allowance_before, U256::zero());

    let approve_request_1 =
        make_cep18_approve_request(sender, cep18_token, spender, allowance_amount_1);
    let approve_request_2 =
        make_cep18_approve_request(sender, cep18_token, spender, allowance_amount_2);

    builder.exec(approve_request_1).expect_success().commit();

    {
        let account_1_allowance_after = cep18_check_allowance_of(builder, owner, spender);
        assert_eq!(account_1_allowance_after, allowance_amount_1);

        let total_supply: U256 = builder.get_value(
            EntityAddr::new_smart_contract(cep18_token.value()),
            TOTAL_SUPPLY_KEY,
        );
        let total_supply: U256 = builder.get_value(
            EntityAddr::new_smart_contract(cep18_token.value()),
            TOTAL_SUPPLY_KEY,
        );
        assert_eq!(total_supply, initial_supply);
    }

    // Approve overwrites existing amount rather than increase it

    builder.exec(approve_request_2).expect_success().commit();

    let account_1_allowance_after = cep18_check_allowance_of(builder, owner, spender);
    assert_eq!(account_1_allowance_after, allowance_amount_2);

    // Swap Key::Account into Hash and other way
    let inverted_spender_key = invert_cep18_address(spender);

    let inverted_spender_allowance = cep18_check_allowance_of(builder, owner, inverted_spender_key);
    assert_eq!(inverted_spender_allowance, U256::zero());

    let total_supply: U256 = builder.get_value(
        EntityAddr::new_smart_contract(cep18_token.value()),
        TOTAL_SUPPLY_KEY,
    );
    let total_supply: U256 = builder.get_value(
        EntityAddr::new_smart_contract(cep18_token.value()),
        TOTAL_SUPPLY_KEY,
    );
    assert_eq!(total_supply, initial_supply);
}
