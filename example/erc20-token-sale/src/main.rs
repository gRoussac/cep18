#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use casper_contract::contract_api::{runtime, storage, system};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
// Importing specific Casper types.
use casper_types::contracts::NamedKeys;
use casper_types::{runtime_args, Parameter, RuntimeArgs, URef};
use casper_types::{
    ApiError, CLType, CLValue, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Key,
};
use erc20_token_sale::constants::{
    COUNT_INVESTMENTS_KEY, COUNT_INVESTORS_KEY, DEPOSIT_PURSE, ENTRY_POINT_COUNTERS,
    ENTRY_POINT_INIT, ENTRY_POINT_INVEST, INVEST_ACCOUNT_KEY, LEDGER, ONE,
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
}

// This is the invest entry point. When called, it records the caller's account
// hash and returns the deposit purse, with add access, to the immediate caller.
#[no_mangle]
pub extern "C" fn invest() {
    let investing_account_key: Key = runtime::get_named_arg(INVEST_ACCOUNT_KEY);
    if let Key::Account(investing_account_hash) = investing_account_key {
        update_ledger_record(investing_account_hash.to_string())
    } else {
        runtime::revert(InvestingError::Test)
    }

    let deposit_purse = get_key_uref(
        DEPOSIT_PURSE,
        InvestingError::MissingDepositPurseURef.into(),
    );
    // The return value is the deposit_purse URef with `add` access only. As a result
    // the entity receiving this purse URef may only add to the purse, and cannot remove
    // funds.
    let value = CLValue::from_t(deposit_purse.into_add()).unwrap_or_revert();
    runtime::ret(value)
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
        vec![Parameter::new(INVEST_ACCOUNT_KEY, CLType::Key)],
        CLType::URef,
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

#[no_mangle]
fn call() {
    // Create names keys for this contract
    let named_keys = create_invest_keys();

    // Create entry points for this contract
    let entry_points = create_entry_points();

    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(TOKEN_SALE_CONTRACT_PKG_HASH.into()),
        Some(TOKEN_SALE_CONTRACT_PKG_UREF.into()),
    );

    runtime::put_key(TOKEN_SALE_CONTRACT_HASH, contract_hash.into());
    let version_uref = storage::new_uref(contract_version);
    runtime::put_key(TOKEN_SALE_CONTRACT_VERSION_KEY, version_uref.into());

    // Call the init entry point to setup and create the deposit purse
    // and the ledger to track purchase made.
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

fn get_key_uref(key: &str, error_key: ApiError) -> casper_types::URef {
    runtime::get_key(key)
        .unwrap_or_revert_with(error_key)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant)
}

fn counter_inc(key: &str, error_key: InvestingError) {
    storage::add(get_key_uref(key, error_key.into()), ONE);
}
