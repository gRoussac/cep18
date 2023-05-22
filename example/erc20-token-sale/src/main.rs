#![no_std]
#![no_main]
extern crate alloc;
pub mod mods;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use casper_contract::contract_api::account::get_main_purse;
use casper_contract::contract_api::runtime::revert;
use casper_contract::contract_api::system::{create_purse, transfer_from_purse_to_purse};
use casper_contract::contract_api::{runtime, storage};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_erc20::constants::{ERC20_TOKEN_CONTRACT_KEY_NAME, OWNER_RUNTIME_ARG_NAME};
use casper_types::account::AccountHash;
use casper_types::contracts::NamedKeys;
use casper_types::system::standard_payment::ARG_AMOUNT;
use casper_types::{runtime_args, ApiError, Parameter, RuntimeArgs, URef, U512};
use casper_types::{CLType, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Key};
use mods::constants::{
    COUNT_INVESTMENTS_KEY, COUNT_INVESTORS_KEY, DEPOSIT_PURSE, ENTRY_POINT_INIT,
    ENTRY_POINT_INVEST, LEDGER, PURSE_NAME_VALUE, TOKEN_PRICE_IN_CSPR, TOKEN_SALE_CONTRACT_HASH,
    TOKEN_SALE_CONTRACT_PKG_HASH, TOKEN_SALE_CONTRACT_PKG_UREF, TOKEN_SALE_CONTRACT_VERSION_KEY,
    ZERO,
};
use mods::utils::{_get_owner_hash, get_key_uref, update_ledger_record};
use mods::InvestingError;

#[no_mangle]
pub extern "C" fn init() {
    if runtime::get_key(DEPOSIT_PURSE).is_none() {
        let deposit_purse = create_purse();
        runtime::put_key(DEPOSIT_PURSE, deposit_purse.into());
        // Create a dictionary to track the mapping of account hashes and the investment made.
        storage::new_dictionary(LEDGER).unwrap_or_revert();
    }
}

// This is the invest entry point. When called, it records the caller's account
// hash and returns the deposit purse, with add access, to the immediate caller.
#[no_mangle]
pub extern "C" fn invest() {
    update_ledger_record(&runtime::get_caller().to_string());

    let investing_amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    let bidder_purse: URef = match runtime::get_key(PURSE_NAME_VALUE) {
        Some(existing_purse) => existing_purse.into_uref().unwrap_or_revert(),
        None => {
            let new_purse = create_purse();
            runtime::put_key(PURSE_NAME_VALUE, new_purse.into());
            new_purse
        }
    };

    transfer_from_purse_to_purse(get_main_purse(), bidder_purse, investing_amount, None)
        .unwrap_or_revert();

    let bidder_purse_out = bidder_purse.into_read_write();
    if !bidder_purse_out.is_writeable() || !bidder_purse_out.is_readable() {
        revert(ApiError::User(101)); // Todo
    }

    let investing_purse = get_key_uref(
        DEPOSIT_PURSE,
        InvestingError::MissingDepositPurseURef.into(),
    );

    let investing_purse_in = investing_purse.into_read_write();
    if !investing_purse_in.is_addable() {
        revert(ApiError::User(101)); // Todo
    }

    transfer_from_purse_to_purse(bidder_purse_out, investing_purse_in, investing_amount, None)
        .unwrap_or_revert();

    let token_amount = investing_amount.as_u64() / TOKEN_PRICE_IN_CSPR;
    let owner: AccountHash = _get_owner_hash();
    let spender: AccountHash = runtime::get_caller();

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
fn create_named_keys() -> BTreeMap<String, Key> {
    // In the named keys of the contract, add a key for the counters
    let mut named_keys = NamedKeys::new();
    for key in [COUNT_INVESTORS_KEY, COUNT_INVESTMENTS_KEY] {
        // Initialize the count to 0 locally
        named_keys.insert(key.into(), storage::new_uref(ZERO).into());
    }
    // insert token contract in name keys
    let token_key_hash = runtime::get_key(ERC20_TOKEN_CONTRACT_KEY_NAME)
        .unwrap_or_revert_with(InvestingError::MissingERC20TokenURef);
    named_keys.insert(ERC20_TOKEN_CONTRACT_KEY_NAME.into(), token_key_hash);
    // Set named key for owner of the token_sale
    named_keys.insert(OWNER_RUNTIME_ARG_NAME.into(), runtime::get_caller().into());
    named_keys
}

#[no_mangle]
fn create_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();

    // This establishes the `invest` entry point for callers looking to invest.
    entry_points.add_entry_point(EntryPoint::new(
        ENTRY_POINT_INVEST,
        vec![Parameter::new(ARG_AMOUNT, CLType::U512)],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

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
    let named_keys = create_named_keys();
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
