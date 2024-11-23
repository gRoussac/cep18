#![no_std]
#![no_main]

extern crate alloc;

use crate::alloc::string::ToString;
use alloc::{boxed::Box, string::String, vec};
use casper_contract::{
    self,
    contract_api::{
        runtime::{self, call_contract, get_key, get_named_arg, put_key, ret},
        storage,
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    bytesrepr::{Bytes, ToBytes},
    runtime_args, ApiError, CLType, CLTyped, CLValue, ContractHash, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, Key, Parameter, RuntimeArgs, U256,
};
use cep18_test_contract::constants::{
    ARG_FILTER_CONTRACT_RETURN_VALUE, ARG_TOKEN_CONTRACT, CEP18_TEST_CONTRACT_NAME,
    CEP18_TEST_CONTRACT_PACKAGE_NAME, ENTRY_POINT_APPROVE_AS_STORED_CONTRACT,
    ENTRY_POINT_CHECK_ALLOWANCE_OF, ENTRY_POINT_CHECK_BALANCE_OF, ENTRY_POINT_CHECK_TOTAL_SUPPLY,
    ENTRY_POINT_SET_FILTER_CONTRACT_RETURN_VALUE, ENTRY_POINT_TRANSFER_AS_STORED_CONTRACT,
    ENTRY_POINT_TRANSFER_FILTER_METHOD, ENTRY_POINT_TRANSFER_FROM_AS_STORED_CONTRACT, RESULT_KEY,
};
use cowl_cep18::{
    constants::{
        ARG_ADDRESS, ARG_AMOUNT, ARG_DATA, ARG_FROM, ARG_OPERATOR, ARG_OWNER, ARG_RECIPIENT,
        ARG_SPENDER, ARG_TO, ENTRY_POINT_ALLOWANCE, ENTRY_POINT_APPROVE, ENTRY_POINT_BALANCE_OF,
        ENTRY_POINT_INIT, ENTRY_POINT_TOTAL_SUPPLY, ENTRY_POINT_TRANSFER,
        ENTRY_POINT_TRANSFER_FROM,
    },
    modalities::TransferFilterContractResult,
};

fn store_result<T: CLTyped + ToBytes>(result: T) {
    match runtime::get_key(RESULT_KEY) {
        Some(Key::URef(uref)) => storage::write(uref, result),
        Some(_) => unreachable!(),
        None => {
            let new_uref = storage::new_uref(result);
            runtime::put_key(RESULT_KEY, new_uref.into());
        }
    }
}

#[no_mangle]
extern "C" fn check_total_supply() {
    let token_contract: ContractHash = ContractHash::new(
        runtime::get_named_arg::<Key>(ARG_TOKEN_CONTRACT)
            .into_hash()
            .unwrap_or_revert(),
    );
    let total_supply: U256 = runtime::call_contract(
        token_contract,
        ENTRY_POINT_TOTAL_SUPPLY,
        RuntimeArgs::default(),
    );
    store_result(total_supply);
}

#[no_mangle]
extern "C" fn check_balance_of() {
    let token_contract: ContractHash = ContractHash::new(
        runtime::get_named_arg::<Key>(ARG_TOKEN_CONTRACT)
            .into_hash()
            .unwrap_or_revert(),
    );
    let address: Key = runtime::get_named_arg(ARG_ADDRESS);

    let balance_args = runtime_args! {
        ARG_ADDRESS => address,
    };
    let result: U256 = runtime::call_contract(token_contract, ENTRY_POINT_BALANCE_OF, balance_args);

    store_result(result);
}

#[no_mangle]
extern "C" fn check_allowance_of() {
    let token_contract: ContractHash = ContractHash::new(
        runtime::get_named_arg::<Key>(ARG_TOKEN_CONTRACT)
            .into_hash()
            .unwrap_or_revert(),
    );
    let owner: Key = runtime::get_named_arg(ARG_OWNER);
    let spender: Key = runtime::get_named_arg(ARG_SPENDER);

    let allowance_args = runtime_args! {
        ARG_OWNER => owner,
        ARG_SPENDER => spender,
    };
    let result: U256 =
        runtime::call_contract(token_contract, ENTRY_POINT_ALLOWANCE, allowance_args);

    store_result(result);
}

#[no_mangle]
extern "C" fn transfer_as_stored_contract() {
    let token_contract: ContractHash = ContractHash::new(
        runtime::get_named_arg::<Key>(ARG_TOKEN_CONTRACT)
            .into_hash()
            .unwrap_or_revert(),
    );
    let recipient: Key = runtime::get_named_arg(ARG_RECIPIENT);
    let amount: U256 = runtime::get_named_arg(ARG_AMOUNT);

    let transfer_args = runtime_args! {
        ARG_RECIPIENT => recipient,
        ARG_AMOUNT => amount,
    };

    runtime::call_contract::<()>(token_contract, ENTRY_POINT_TRANSFER, transfer_args);
}

#[no_mangle]
extern "C" fn transfer_from_as_stored_contract() {
    let token_contract: ContractHash = ContractHash::new(
        runtime::get_named_arg::<Key>(ARG_TOKEN_CONTRACT)
            .into_hash()
            .unwrap_or_revert(),
    );
    let owner: Key = runtime::get_named_arg(ARG_OWNER);
    let recipient: Key = runtime::get_named_arg(ARG_RECIPIENT);
    let amount: U256 = runtime::get_named_arg(ARG_AMOUNT);

    let transfer_from_args = runtime_args! {
        ARG_OWNER => owner,
        ARG_RECIPIENT => recipient,
        ARG_AMOUNT => amount,
    };

    runtime::call_contract::<()>(
        token_contract,
        ENTRY_POINT_TRANSFER_FROM,
        transfer_from_args,
    );
}

#[no_mangle]
extern "C" fn approve_as_stored_contract() {
    let token_contract: ContractHash = ContractHash::new(
        runtime::get_named_arg::<Key>(ARG_TOKEN_CONTRACT)
            .into_hash()
            .unwrap_or_revert(),
    );
    let spender: Key = runtime::get_named_arg(ARG_SPENDER);
    let amount: U256 = runtime::get_named_arg(ARG_AMOUNT);

    let approve_args = runtime_args! {
        ARG_SPENDER => spender,
        ARG_AMOUNT => amount,
    };

    runtime::call_contract::<()>(token_contract, ENTRY_POINT_APPROVE, approve_args);
}

#[no_mangle]
pub extern "C" fn call() {
    let mut entry_points = EntryPoints::new();
    let check_total_supply_entrypoint = EntryPoint::new(
        String::from(ENTRY_POINT_CHECK_TOTAL_SUPPLY),
        vec![Parameter::new(ARG_TOKEN_CONTRACT, ContractHash::cl_type())],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );
    let check_balance_of_entrypoint = EntryPoint::new(
        String::from(ENTRY_POINT_CHECK_BALANCE_OF),
        vec![
            Parameter::new(ARG_TOKEN_CONTRACT, ContractHash::cl_type()),
            Parameter::new(ARG_ADDRESS, Key::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );
    let check_allowance_of_entrypoint = EntryPoint::new(
        String::from(ENTRY_POINT_CHECK_ALLOWANCE_OF),
        vec![
            Parameter::new(ARG_TOKEN_CONTRACT, ContractHash::cl_type()),
            Parameter::new(ARG_OWNER, Key::cl_type()),
            Parameter::new(ARG_SPENDER, Key::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let transfer_as_stored_contract_entrypoint = EntryPoint::new(
        String::from(ENTRY_POINT_TRANSFER_AS_STORED_CONTRACT),
        vec![
            Parameter::new(ARG_TOKEN_CONTRACT, ContractHash::cl_type()),
            Parameter::new(ARG_RECIPIENT, Key::cl_type()),
            Parameter::new(ARG_AMOUNT, U256::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let approve_as_stored_contract_entrypoint = EntryPoint::new(
        String::from(ENTRY_POINT_APPROVE_AS_STORED_CONTRACT),
        vec![
            Parameter::new(ARG_TOKEN_CONTRACT, ContractHash::cl_type()),
            Parameter::new(ARG_SPENDER, Key::cl_type()),
            Parameter::new(ARG_AMOUNT, U256::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let transfer_from_as_stored_contract_entrypoint = EntryPoint::new(
        String::from(ENTRY_POINT_TRANSFER_FROM_AS_STORED_CONTRACT),
        vec![
            Parameter::new(ARG_TOKEN_CONTRACT, ContractHash::cl_type()),
            Parameter::new(ARG_OWNER, Key::cl_type()),
            Parameter::new(ARG_RECIPIENT, Key::cl_type()),
            Parameter::new(ARG_AMOUNT, U256::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    /* COWL */
    let init = EntryPoint::new(
        ENTRY_POINT_INIT,
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    entry_points.add_entry_point(init);

    let can_transfer = EntryPoint::new(
        ENTRY_POINT_TRANSFER_FILTER_METHOD,
        vec![
            Parameter::new(ARG_OPERATOR, CLType::Key),
            Parameter::new(ARG_FROM, CLType::Key),
            Parameter::new(ARG_TO, CLType::Key),
            Parameter::new(ARG_AMOUNT, CLType::List(Box::new(CLType::U256))),
            Parameter::new(ARG_DATA, CLType::Option(Box::new(Bytes::cl_type()))),
        ],
        TransferFilterContractResult::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    entry_points.add_entry_point(can_transfer);

    let set_filter_contract_return_value = EntryPoint::new(
        ENTRY_POINT_SET_FILTER_CONTRACT_RETURN_VALUE,
        vec![Parameter::new(
            ARG_FILTER_CONTRACT_RETURN_VALUE,
            TransferFilterContractResult::cl_type(),
        )],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    entry_points.add_entry_point(set_filter_contract_return_value);
    /*  */

    entry_points.add_entry_point(check_total_supply_entrypoint);
    entry_points.add_entry_point(check_balance_of_entrypoint);
    entry_points.add_entry_point(check_allowance_of_entrypoint);
    entry_points.add_entry_point(transfer_as_stored_contract_entrypoint);
    entry_points.add_entry_point(approve_as_stored_contract_entrypoint);
    entry_points.add_entry_point(transfer_from_as_stored_contract_entrypoint);

    let (contract_hash, _version) = storage::new_contract(
        entry_points,
        None,
        Some(CEP18_TEST_CONTRACT_PACKAGE_NAME.to_string()),
        None,
    );

    put_key(CEP18_TEST_CONTRACT_NAME, Key::from(contract_hash));

    /* COWL */
    let token_contract = get_named_arg::<Key>(ARG_TOKEN_CONTRACT);
    // Call contract to initialize it
    let init_args = runtime_args! {
        ARG_TOKEN_CONTRACT => token_contract,
    };
    call_contract::<()>(contract_hash, ENTRY_POINT_INIT, init_args);
    /*  */
}

/* COWL */
#[no_mangle]
pub extern "C" fn init() {
    let token_contract = get_named_arg::<Key>(ARG_TOKEN_CONTRACT);
    put_key(ARG_TOKEN_CONTRACT, token_contract);
}

// Update stored value for as a contract filter result value
#[no_mangle]
pub extern "C" fn set_filter_contract_return_value() {
    let value: TransferFilterContractResult = get_named_arg(ARG_FILTER_CONTRACT_RETURN_VALUE);
    let uref = storage::new_uref(value);
    put_key(ARG_FILTER_CONTRACT_RETURN_VALUE, uref.into());
}

// Check that some values are sent by token contract and return a TransferFilterContractResult
#[no_mangle]
pub extern "C" fn can_transfer() {
    let _operator: Key = get_named_arg(ARG_OPERATOR);
    let _from: Key = get_named_arg(ARG_FROM);
    let _to: Key = get_named_arg(ARG_TO);
    let _amount: U256 = get_named_arg(ARG_AMOUNT);
    let _data: Option<Bytes> = get_named_arg(ARG_DATA);

    let key = get_key(ARG_FILTER_CONTRACT_RETURN_VALUE);
    if key.is_none() {
        ret(CLValue::from_t(TransferFilterContractResult::DenyTransfer).unwrap_or_revert());
    }
    let uref = get_key(ARG_FILTER_CONTRACT_RETURN_VALUE)
        .unwrap_or_revert()
        .into_uref();
    let value: TransferFilterContractResult =
        storage::read(uref.unwrap_or_revert_with(ApiError::ValueNotFound))
            .unwrap_or_revert()
            .unwrap_or(TransferFilterContractResult::DenyTransfer);
    ret(CLValue::from_t(value).unwrap_or_revert());
}
/*  */
