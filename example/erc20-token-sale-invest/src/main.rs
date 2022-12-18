#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");
extern crate alloc;

use casper_contract::{
    contract_api::{account, runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{runtime_args, ApiError, ContractHash, Key, RuntimeArgs, URef, U512};
use erc20_token_sale::constants::{
    ENTRY_POINT_INVEST, INVEST_ACCOUNT_KEY, TOKEN_SALE_CONTRACT_HASH,
};
const INVESTING_AMOUNT: &str = "amount";

#[no_mangle]
pub extern "C" fn call() {
    let contract_hash = {
        let token_sale_uref =
            runtime::get_key(TOKEN_SALE_CONTRACT_HASH).unwrap_or_revert_with(ApiError::GetKey);
        if let Key::Hash(hash) = token_sale_uref {
            ContractHash::new(hash)
        } else {
            runtime::revert(ApiError::User(66)); // TODO
        }
    };
    let investing_account_key: Key = runtime::get_named_arg(INVEST_ACCOUNT_KEY);
    let investing_amount: U512 = runtime::get_named_arg(INVESTING_AMOUNT);

    runtime::call_contract::<()>(
        contract_hash,
        ENTRY_POINT_INVEST,
        runtime_args! {
            INVEST_ACCOUNT_KEY => investing_account_key,
            INVESTING_AMOUNT => investing_amount,
        },
    );
}
