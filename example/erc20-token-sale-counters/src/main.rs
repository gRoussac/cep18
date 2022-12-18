#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");
extern crate alloc;

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{ApiError, CLValue, ContractHash, Key, RuntimeArgs};
use erc20_token_sale::constants::{ENTRY_POINT_COUNTERS, TOKEN_SALE_CONTRACT_HASH};

#[no_mangle]
pub extern "C" fn call() {
    let contract_hash = {
        let counter_uref =
            runtime::get_key(TOKEN_SALE_CONTRACT_HASH).unwrap_or_revert_with(ApiError::GetKey);
        if let Key::Hash(hash) = counter_uref {
            ContractHash::new(hash)
        } else {
            runtime::revert(ApiError::User(66)); // 66 ?
        }
    };

    let counters_value: (u64, u64) =
        runtime::call_contract(contract_hash, ENTRY_POINT_COUNTERS, RuntimeArgs::new());

    // The return value of a directly deployed contract is never used.
    // runtime::ret(CLValue::from_t(counters_value).unwrap_or_revert())
    // Seting a new key instead
    let counters_uref = storage::new_uref(counters_value);
    runtime::put_key(ENTRY_POINT_COUNTERS.into(), counters_uref.into())
}

// // This entry point returns the counters
// #[no_mangle]
// pub extern "C" fn get_counters() {
//     let uref = get_key_uref(COUNT_INVESTORS_KEY, ApiError::MissingKey);
//     let count_investors: u64 = storage::read(uref)
//         .unwrap_or_revert_with(ApiError::Read)
//         .unwrap_or_revert_with(ApiError::ValueNotFound);
//     let uref = get_key_uref(COUNT_INVESTMENTS_KEY, ApiError::MissingKey);
//     let count_investments: u64 = storage::read(uref)
//         .unwrap_or_revert_with(ApiError::Read)
//         .unwrap_or_revert_with(ApiError::ValueNotFound);
//     let typed_result = CLValue::from_t((count_investors, count_investments)).unwrap_or_revert();
//     runtime::ret(typed_result); // return the counters value as CL value tuple
// }
