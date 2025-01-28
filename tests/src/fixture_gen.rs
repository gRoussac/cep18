use casper_engine_test_support::utils::create_run_genesis_request;
use casper_engine_test_support::{
    ExecuteRequestBuilder, LmdbWasmTestBuilder, DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_PUBLIC_KEY,
};
use casper_fixtures::generate_fixture;
use casper_types::{
    account::AccountHash,
    addressable_entity::{AddressableEntityHash, EntityKindTag},
    bytesrepr::FromBytes,
    runtime_args, CLTyped, EntityAddr, GenesisAccount, Key, Motes, PackageHash, RuntimeArgs, U256,
    U512, {PublicKey, SecretKey},
};
use cep18::constants::{
    ARG_ADDRESS, ARG_AMOUNT, ARG_DECIMALS, ARG_NAME, ARG_OWNER, ARG_SYMBOL, ARG_TOTAL_SUPPLY,
    ENTRY_POINT_BALANCE_OF,
};
use cep18_test_contract::constants::{
    ARG_TOKEN_CONTRACT, CEP18_TEST_CONTRACT_PACKAGE_NAME, RESULT_KEY,
};
use once_cell::sync::Lazy;
use utility::constants::{
    CEP18_CONTRACT_WASM, CEP18_TEST_CONTRACT_WASM, CEP18_TEST_TOKEN_CONTRACT_NAME, TOKEN_DECIMALS,
    TOKEN_NAME, TOKEN_SYMBOL, TOKEN_TOTAL_SUPPLY,
};

#[allow(unused)]
mod utility;

pub static ACCOUNT_1_SECRET_KEY: Lazy<SecretKey> =
    Lazy::new(|| SecretKey::secp256k1_from_bytes([221u8; 32]).unwrap());
pub static ACCOUNT_1_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| PublicKey::from(&*ACCOUNT_1_SECRET_KEY));
pub static ACCOUNT_1_ADDR: Lazy<AccountHash> = Lazy::new(|| ACCOUNT_1_PUBLIC_KEY.to_account_hash());

pub static ACCOUNT_2_SECRET_KEY: Lazy<SecretKey> =
    Lazy::new(|| SecretKey::secp256k1_from_bytes([212u8; 32]).unwrap());
pub static ACCOUNT_2_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| PublicKey::from(&*ACCOUNT_2_SECRET_KEY));
pub static ACCOUNT_2_ADDR: Lazy<AccountHash> = Lazy::new(|| ACCOUNT_2_PUBLIC_KEY.to_account_hash());

pub const AMOUNT_TRANSFER_1: u64 = 200_001;
pub const AMOUNT_TRANSFER_2: u64 = 19_999;
pub const AMOUNT_ALLOWANCE_1: u64 = 456_789;
pub const AMOUNT_ALLOWANCE_2: u64 = 87_654;

pub const METHOD_TRANSFER_AS_STORED_CONTRACT: &str = "transfer_as_stored_contract";
pub const METHOD_APPROVE_AS_STORED_CONTRACT: &str = "approve_as_stored_contract";
pub const METHOD_FROM_AS_STORED_CONTRACT: &str = "transfer_from_as_stored_contract";

pub const TOKEN_OWNER_ADDRESS_1: Key = Key::Account(AccountHash::new([42; 32]));
pub const AMOUNT_1: u64 = 1_000_000;
pub const TOKEN_OWNER_ADDRESS_2: Key = Key::Hash([42; 32]);
pub const AMOUNT_2: u64 = 2_000_000;

pub const ENTRY_POINT_MINT: &str = "mint";
pub const METHOD_BURN: &str = "burn";
pub const DECREASE_ALLOWANCE: &str = "decrease_allowance";
pub const INCREASE_ALLOWANCE: &str = "increase_allowance";
pub const ENABLE_MINT_BURN: &str = "enable_mint_burn";
pub const ADMIN_LIST: &str = "admin_list";
pub const MINTER_LIST: &str = "minter_list";
pub const NONE_LIST: &str = "none_list";
pub const CHANGE_SECURITY: &str = "change_security";

pub(crate) fn get_test_result<T: FromBytes + CLTyped>(
    builder: &mut LmdbWasmTestBuilder,
    cep18_test_contract_package: PackageHash,
) -> T {
    let contract_package = builder
        .get_package(cep18_test_contract_package)
        .expect("should have contract package");
    let enabled_versions = contract_package.enabled_versions();

    let contract_hash = enabled_versions
        .contract_hashes()
        .last()
        .expect("should have latest version");
    let entity_addr = EntityAddr::new_smart_contract(contract_hash.value());
    builder.get_value(entity_addr, RESULT_KEY)
}

pub(crate) fn cep18_check_balance_of(
    builder: &mut LmdbWasmTestBuilder,
    cep18_contract_hash: &AddressableEntityHash,
    address: Key,
) -> U256 {
    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let cep18_test_contract_package = account
        .named_keys()
        .get(CEP18_TEST_CONTRACT_PACKAGE_NAME)
        .and_then(|key| key.into_package_hash())
        .expect("should have test contract hash");

    let check_balance_args = runtime_args! {
        ARG_TOKEN_CONTRACT => Key::addressable_entity_key(EntityKindTag::SmartContract, *cep18_contract_hash),
        ARG_ADDRESS => address,
    };
    let exec_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_test_contract_package,
        None,
        ENTRY_POINT_BALANCE_OF,
        check_balance_args,
    )
    .build();
    builder.exec(exec_request).expect_success().commit();

    get_test_result(builder, cep18_test_contract_package)
}

fn main() {
    let genesis_request = create_run_genesis_request(vec![
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
    ]);
    generate_fixture("cep18-2.0.0-rc3-minted", genesis_request, |builder| {
        let mint_amount = U256::one();

        let install_args = runtime_args! {
            ARG_NAME => TOKEN_NAME,
            ARG_SYMBOL => TOKEN_SYMBOL,
            ARG_DECIMALS => TOKEN_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
            ENABLE_MINT_BURN => true,
        };
        let install_request_1 = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            CEP18_CONTRACT_WASM,
            install_args,
        )
        .build();

        let install_request_2 = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            CEP18_TEST_CONTRACT_WASM,
            RuntimeArgs::default(),
        )
        .build();

        builder.exec(install_request_1).expect_success().commit();
        builder.exec(install_request_2).expect_success().commit();

        let account = builder
            .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
            .unwrap();
        let account_named_keys = account.named_keys();

        let cep18_contract_hash = account_named_keys
            .get(CEP18_TEST_TOKEN_CONTRACT_NAME)
            .and_then(|key| key.into_entity_hash())
            .expect("should have contract hash");

        let addressable_cep18_contract_hash =
            AddressableEntityHash::new(cep18_contract_hash.value());
        let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            addressable_cep18_contract_hash,
            ENTRY_POINT_MINT,
            runtime_args! {ARG_OWNER => TOKEN_OWNER_ADDRESS_1, ARG_AMOUNT => U256::from(AMOUNT_1)},
        )
        .build();
        builder.exec(mint_request).expect_success().commit();
        let mint_request_2 = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            addressable_cep18_contract_hash,
            ENTRY_POINT_MINT,
            runtime_args! {ARG_OWNER => TOKEN_OWNER_ADDRESS_2, ARG_AMOUNT => U256::from(AMOUNT_2)},
        )
        .build();
        builder.exec(mint_request_2).expect_success().commit();
        assert_eq!(
            cep18_check_balance_of(
                builder,
                &cep18_contract_hash,
                Key::AddressableEntity(casper_types::EntityAddr::Account(
                    DEFAULT_ACCOUNT_ADDR.value()
                ))
            ),
            U256::from(TOKEN_TOTAL_SUPPLY),
        );
        assert_eq!(
            cep18_check_balance_of(builder, &cep18_contract_hash, TOKEN_OWNER_ADDRESS_1),
            U256::from(AMOUNT_1)
        );
        assert_eq!(
            cep18_check_balance_of(builder, &cep18_contract_hash, TOKEN_OWNER_ADDRESS_2),
            U256::from(AMOUNT_2)
        );

        let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            addressable_cep18_contract_hash,
            ENTRY_POINT_MINT,
            runtime_args! {
                ARG_OWNER => TOKEN_OWNER_ADDRESS_1,
                ARG_AMOUNT => mint_amount,
            },
        )
        .build();

        builder.exec(mint_request).expect_success().commit();

        assert_eq!(
            cep18_check_balance_of(builder, &cep18_contract_hash, TOKEN_OWNER_ADDRESS_1),
            U256::from(AMOUNT_1) + mint_amount,
        );
        assert_eq!(
            cep18_check_balance_of(builder, &cep18_contract_hash, TOKEN_OWNER_ADDRESS_2),
            U256::from(AMOUNT_2)
        );
    })
    .unwrap();
}
