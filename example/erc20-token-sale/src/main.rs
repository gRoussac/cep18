#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");
extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;

use casper_contract::contract_api::{runtime, storage};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
// Importing specific Casper types.
use casper_erc20::ERC20;
use casper_types::U256;
use casper_types::{
    ApiError, CLType, CLValue, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Key,
    Parameter,
};

// Creating constants for the various contract entry points.
const ENTRY_POINT_INIT: &str = "init";
const ENTRY_POINT_INVEST: &str = "invest";

// Creating constants for values within the contract.
const INVEST_ACCOUNT_KEY: &str = "invest_account_key";
const LEDGER: &str = "ledger";
const TOKEN_PRICE_IN_CSPR: U256 = U256::from(1);
const DEPOSIT_PURSE: &str = "deposit_purse";

enum InvestingError {
    InvalidKeyVariant,
    MissingDepositPurseURef,
    MissingLedgerSeedURef,
}

impl From<InvestingError> for ApiError {
    fn from(code: InvestingError) -> Self {
        ApiError::User(code as u16)
    }
}

#[no_mangle]
pub extern "C" fn init() {
    // Create a dictionary to track the mapping of account hashes the investment made.
    storage::new_dictionary(LEDGER).unwrap_or_revert();
}

// This is the invest entry point. When called, it records the caller's account
// hash and returns the deposit purse, with add access, to the immediate caller.
#[no_mangle]
pub extern "C" fn invest() {
    let donating_account_key: Key = runtime::get_named_arg(INVEST_ACCOUNT_KEY);
    if let Key::Account(donating_account_hash) = donating_account_key {
        update_ledger_record(donating_account_hash.to_string())
    } else {
        runtime::revert(InvestingError::InvalidKeyVariant)
    }
    let donation_purse = *runtime::get_key(DEPOSIT_PURSE)
        .unwrap_or_revert_with(InvestingError::MissingDepositPurseURef)
        .as_uref()
        .unwrap_or_revert();
    // The return value is the donation_purse URef with `add` access only. As a result
    // the entity receiving this purse URef may only add to the purse, and cannot remove
    // funds.
    let value = CLValue::from_t(donation_purse.into_add()).unwrap_or_revert();
    runtime::ret(value)
}

#[no_mangle]
fn call() {
    // let _token = ERC20::from().unwrap_or_revert();

    // This establishes the `init` entry point for initializing the contract's infrastructure.
    let init_entry_point = EntryPoint::new(
        ENTRY_POINT_INIT,
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    // This establishes the `invest` entry point for callers looking to invest.
    let invest_entry_point = EntryPoint::new(
        ENTRY_POINT_INVEST,
        vec![Parameter::new(INVEST_ACCOUNT_KEY, CLType::Key)],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(init_entry_point);
    entry_points.add_entry_point(invest_entry_point);
}

fn update_ledger_record(dictionary_item_key: String) {
    // Acquiring the LEDGER seed URef to properly assign the dictionary item.
    let ledger_seed_uref = *runtime::get_key(LEDGER)
        .unwrap_or_revert_with(InvestingError::MissingLedgerSeedURef)
        .as_uref()
        .unwrap_or_revert();

    // This identifies an item within the dictionary and either creates or updates the associated value.
    match storage::dictionary_get::<u64>(ledger_seed_uref, &dictionary_item_key).unwrap_or_revert()
    {
        None => storage::dictionary_put(ledger_seed_uref, &dictionary_item_key, 1u64),
        Some(current_number_of_donations) => storage::dictionary_put(
            ledger_seed_uref,
            &dictionary_item_key,
            current_number_of_donations + 1u64,
        ),
    }
}
