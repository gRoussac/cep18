#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use casper_contract::contract_api::{account, runtime, storage, system};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_erc20::constants::{
    ADDRESS_RUNTIME_ARG_NAME, AMOUNT_RUNTIME_ARG_NAME, APPROVE_ENTRY_POINT_NAME, BALANCES_KEY_NAME,
    BALANCE_OF_ENTRY_POINT_NAME, ERC20_TOKEN_CONTRACT_KEY_NAME, OWNER_RUNTIME_ARG_NAME,
    RECIPIENT_RUNTIME_ARG_NAME, SPENDER_RUNTIME_ARG_NAME, TRANSFER_ENTRY_POINT_NAME,
    TRANSFER_FROM_ENTRY_POINT_NAME,
};
use casper_erc20::Address;
use casper_types::account::AccountHash;
use casper_types::bytesrepr::ToBytes;
// Importing specific Casper types.
use casper_types::contracts::NamedKeys;
use casper_types::{
    runtime_args, ContractHash, ContractPackageHash, Parameter, RuntimeArgs, U256, U512,
};
use casper_types::{
    ApiError, CLType, CLValue, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Key,
};
use erc20_token_sale::constants::{
    COUNT_INVESTMENTS_KEY, COUNT_INVESTORS_KEY, DEPOSIT_PURSE, ENTRY_POINT_COUNTERS,
    ENTRY_POINT_INIT, ENTRY_POINT_INVEST, INVESTING_AMOUNT, LEDGER, ONE, TOKEN_PRICE_IN_CSPR,
    TOKEN_SALE_CONTRACT_HASH, TOKEN_SALE_CONTRACT_PKG_HASH, TOKEN_SALE_CONTRACT_PKG_UREF,
    TOKEN_SALE_CONTRACT_VERSION_KEY, ZERO,
};
use erc20_token_sale::InvestingError;

#[no_mangle]
pub extern "C" fn init() {
    let deposit_purse = system::create_purse();
    runtime::put_key(DEPOSIT_PURSE, deposit_purse.into());

    // Create a dictionary to track the mapping of account hashes and the investment made.
    storage::new_dictionary(LEDGER).unwrap_or_revert();
    storage::new_dictionary(BALANCES_KEY_NAME).unwrap_or_revert();
}

// This is the invest entry point. When called, it records the caller's account
// hash and returns the deposit purse, with add access, to the immediate caller.
#[no_mangle]
pub extern "C" fn invest() {
    // TODO check caller?
    update_ledger_record(runtime::get_caller().to_string());

    // let investing_amount: U512 = runtime::get_named_arg(INVESTING_AMOUNT);
    // let investing_purse_uref = get_key_uref(
    //     DEPOSIT_PURSE,
    //     InvestingError::MissingDepositPurseURef.into(),
    // );
    // system::transfer_from_purse_to_purse(
    //     account::get_main_purse(),
    //     investing_purse_uref.into_add(),
    //     investing_amount,
    //     None,
    // )
    // .unwrap_or_revert();

    // let token_amount = investing_amount.as_u64() / TOKEN_PRICE_IN_CSPR;
    // let owner: AccountHash = get_owner_hash();
    // let spender: AccountHash = runtime::get_caller();

    // // Allowance with approve
    // runtime::call_contract::<U256>(
    //     get_token_hash(),
    //     APPROVE_ENTRY_POINT_NAME,
    //     runtime_args! {
    //         SPENDER_RUNTIME_ARG_NAME => Address::from(spender),
    //         AMOUNT_RUNTIME_ARG_NAME => U256::from(token_amount)
    //     },
    // );

    // // TransferFrom
    // runtime::call_contract::<U256>(
    //     get_token_hash(),
    //     TRANSFER_FROM_ENTRY_POINT_NAME,
    //     runtime_args! {
    //         OWNER_RUNTIME_ARG_NAME => Address::from(owner),
    //         RECIPIENT_RUNTIME_ARG_NAME => Address::from(spender),
    //         AMOUNT_RUNTIME_ARG_NAME => U256::from(token_amount)
    //     },
    // );
    // update_balance(runtime::get_caller().to_string(), token_amount);
}

#[no_mangle]
pub extern "C" fn balance_of() {
    let balance_of_tokens: U256 = runtime::call_contract::<U256>(
        get_token_hash(),
        BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args! {
            ADDRESS_RUNTIME_ARG_NAME => Address::from(runtime::get_caller()),
        },
    );
    runtime::ret(CLValue::from_t(balance_of_tokens).unwrap_or_revert())
}

// This entry point returns the counters
#[no_mangle]
pub extern "C" fn get_counters() {
    let uref = get_key_uref(COUNT_INVESTORS_KEY, ApiError::MissingKey);
    let count_investors: u64 = storage::read(uref)
        .unwrap_or_revert_with(ApiError::Read)
        .unwrap_or_revert_with(ApiError::ValueNotFound);
    let uref = get_key_uref(COUNT_INVESTMENTS_KEY, ApiError::MissingKey);
    let count_investments: u64 = storage::read(uref)
        .unwrap_or_revert_with(ApiError::Read)
        .unwrap_or_revert_with(ApiError::ValueNotFound);
    let typed_result = CLValue::from_t((count_investors, count_investments)).unwrap_or_revert();
    runtime::ret(typed_result); // return the counters value as CL value tuple
}

#[no_mangle]
fn create_invest_keys() -> BTreeMap<String, Key> {
    // In the named keys of the contract, add a key for the counters
    let mut named_keys = NamedKeys::new();
    for key in [COUNT_INVESTORS_KEY, COUNT_INVESTMENTS_KEY] {
        // Initialize the count to 0 locally
        named_keys.insert(key.into(), storage::new_uref(ZERO).into());
    }
    named_keys
}

#[no_mangle]
fn create_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();

    // This establishes the `invest` entry point for callers looking to invest.
    entry_points.add_entry_point(EntryPoint::new(
        ENTRY_POINT_INVEST,
        vec![Parameter::new(INVESTING_AMOUNT, CLType::U512)],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // This establishes the `balance_of` entry point.
    entry_points.add_entry_point(EntryPoint::new(
        BALANCE_OF_ENTRY_POINT_NAME,
        vec![],
        CLType::U256,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // This establishes the `get counters` entry point
    entry_points.add_entry_point(EntryPoint::new(
        ENTRY_POINT_COUNTERS,
        vec![],
        CLType::I32,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // TODO check re-calling init as entry point
    // This establishes the `init` entry point for initializing the contract's infrastructure.
    let init_entry_point = EntryPoint::new(
        ENTRY_POINT_INIT,
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );
    entry_points.add_entry_point(init_entry_point);
    entry_points
}

fn make_dictionary_item_key(owner: Address) -> String {
    let preimage = owner.to_bytes().unwrap_or_revert();
    base64::encode(&preimage)
}

#[no_mangle]
fn call() {
    // Create names keys for this contract
    let mut named_keys = create_invest_keys();
    // insert token contract in name keys
    let token_key_hash = runtime::get_key(ERC20_TOKEN_CONTRACT_KEY_NAME)
        .unwrap_or_revert_with(InvestingError::MissingERC20TokenURef);
    named_keys.insert(ERC20_TOKEN_CONTRACT_KEY_NAME.into(), token_key_hash);
    // Set neamed key for owner of the token_sale
    named_keys.insert(OWNER_RUNTIME_ARG_NAME.into(), runtime::get_caller().into());
    // Create entry points for this contract
    let entry_points = create_entry_points();

    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(TOKEN_SALE_CONTRACT_PKG_HASH.into()),
        Some(TOKEN_SALE_CONTRACT_PKG_UREF.into()),
    );

    // Update
    // let package_hash = {
    //     let package_hash_key =
    //         runtime::get_key(TOKEN_SALE_CONTRACT_PKG_HASH).unwrap_or_revert_with(ApiError::GetKey);
    //     if let Key::Hash(hash) = package_hash_key {
    //         ContractPackageHash::new(hash)
    //     } else {
    //         runtime::revert(ApiError::User(66)); // TODO
    //     }
    // };
    // let (contract_hash, contract_version) =
    //     storage::add_contract_version(package_hash, entry_points, named_keys);

    runtime::put_key(TOKEN_SALE_CONTRACT_HASH, contract_hash.into());
    let version_uref = storage::new_uref(contract_version);
    runtime::put_key(TOKEN_SALE_CONTRACT_VERSION_KEY, version_uref.into());

    // Call the init entry point to setup and create the deposit purse
    // and the ledger to track investments made.
    runtime::call_contract::<()>(contract_hash, ENTRY_POINT_INIT, runtime_args! {})
}

// Update investors ledger and increment counters
fn update_ledger_record(dictionary_item_key: String) {
    // Acquiring the LEDGER seed URef to properly assign the dictionary item.
    let ledger_seed_uref = get_key_uref(LEDGER, InvestingError::MissingLedgerSeedURef.into());
    // This identifies an item within the dictionary and either creates or updates the associated value.
    match storage::dictionary_get::<u64>(ledger_seed_uref, &dictionary_item_key).unwrap_or_revert()
    {
        None => {
            storage::dictionary_put(ledger_seed_uref, &dictionary_item_key, ONE);
            // Update counter for investors
            counter_inc(
                COUNT_INVESTORS_KEY,
                InvestingError::MissingCountInvestorsKey,
            );
            counter_inc(
                COUNT_INVESTMENTS_KEY,
                InvestingError::MissingCountInvestmentsKey,
            );
        }
        Some(current_number_of_purchase) => {
            storage::dictionary_put(
                ledger_seed_uref,
                &dictionary_item_key,
                current_number_of_purchase + ONE,
            );
            counter_inc(
                COUNT_INVESTMENTS_KEY,
                InvestingError::MissingCountInvestmentsKey,
            );
        }
    }
}

// Update investors token balance
fn update_balance(dictionary_item_key: String, dictionary_item_value: u64) {
    // Acquiring the BALANCE seed URef to properly assign the dictionary item.
    let balance_seed_uref = get_key_uref(
        BALANCES_KEY_NAME,
        InvestingError::MissingBalancesSeedURef.into(),
    );
    match storage::dictionary_get::<u64>(balance_seed_uref, &dictionary_item_key).unwrap_or_revert()
    {
        None => {
            storage::dictionary_put(
                balance_seed_uref,
                &dictionary_item_key,
                dictionary_item_value,
            );
        }
        Some(current_token_balance) => {
            storage::dictionary_put(
                balance_seed_uref,
                &dictionary_item_key,
                current_token_balance + dictionary_item_value,
            );
        }
    }
}

fn get_key_uref(key: &str, error_key: ApiError) -> casper_types::URef {
    runtime::get_key(key)
        .unwrap_or_revert_with(error_key)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant)
}

fn counter_inc(key: &str, error_key: InvestingError) {
    storage::add(get_key_uref(key, error_key.into()), ONE);
}

fn get_token_hash() -> ContractHash {
    let contract_hash = {
        let token_key_hash = runtime::get_key(ERC20_TOKEN_CONTRACT_KEY_NAME)
            .unwrap_or_revert_with(InvestingError::MissingERC20TokenURef);
        if let Key::Hash(hash) = token_key_hash {
            ContractHash::new(hash)
        } else {
            runtime::revert(ApiError::User(66)); // TODO
        }
    };
    contract_hash
}

fn get_owner_hash() -> AccountHash {
    let owner_hash = {
        let owner_key_hash = runtime::get_key(OWNER_RUNTIME_ARG_NAME)
            .unwrap_or_revert_with(InvestingError::MissingOwnerHash);
        if let Key::Hash(hash) = owner_key_hash {
            AccountHash::new(hash)
        } else {
            runtime::revert(ApiError::User(66)); // TODO
        }
    };
    owner_hash
}
