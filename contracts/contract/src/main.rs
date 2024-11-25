#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use casper_contract::{
    contract_api::{
        runtime::{
            self, call_versioned_contract, get_caller, get_key, get_named_arg, put_key, revert,
        },
        storage::{self, dictionary_put},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    bytesrepr::{Bytes, ToBytes},
    contracts::NamedKeys,
    runtime_args, CLValue, ContractHash, ContractPackageHash, Key, RuntimeArgs, U256,
};
#[cfg(feature = "contract-support")]
use cowl_cep18::{
    allowances::{get_allowances_uref, read_allowance_from, write_allowance_to},
    balances::{get_balances_uref, read_balance_from, transfer_balance, write_balance_to},
    utils::{
        base64_encode, get_immediate_caller_address, get_optional_named_arg_with_user_errors,
        get_stored_value, get_total_supply_uref, get_transfer_filter_contract_package_hash,
        get_transfer_filter_method, read_total_supply_from, write_total_supply_to,
    },
};
use cowl_cep18::{
    constants::{
        ADMIN_LIST, ARG_ADDRESS, ARG_AMOUNT, ARG_DATA, ARG_DECIMALS, ARG_ENABLE_MINT_BURN,
        ARG_EVENTS_MODE, ARG_FROM, ARG_NAME, ARG_OPERATOR, ARG_OWNER, ARG_PACKAGE_HASH,
        ARG_RECIPIENT, ARG_SPENDER, ARG_SYMBOL, ARG_TO, ARG_TOTAL_SUPPLY,
        ARG_TRANSFER_FILTER_CONTRACT_PACKAGE, ARG_TRANSFER_FILTER_METHOD, DICT_ALLOWANCES,
        DICT_BALANCES, DICT_SECURITY_BADGES, ENTRY_POINT_INIT, ENTRY_POINT_UPGRADE, MINTER_LIST,
        NONE_LIST, PREFIX_ACCESS_KEY_NAME, PREFIX_CEP18, PREFIX_CONTRACT_NAME,
        PREFIX_CONTRACT_PACKAGE_NAME, PREFIX_CONTRACT_VERSION,
    },
    entry_points::generate_entry_points,
    error::Cep18Error,
    events::{
        init_events, record_event_dictionary, Burn, ChangeSecurity, DecreaseAllowance, Event,
        IncreaseAllowance, Mint, SetAllowance, Transfer, TransferFilterUpdate, TransferFrom,
        Upgrade,
    },
    modalities::TransferFilterContractResult,
    security::{change_sec_badge, sec_check, SecurityBadge},
};

#[no_mangle]
pub extern "C" fn name() {
    runtime::ret(CLValue::from_t(get_stored_value::<String>(ARG_NAME)).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn symbol() {
    runtime::ret(CLValue::from_t(get_stored_value::<String>(ARG_SYMBOL)).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn decimals() {
    runtime::ret(CLValue::from_t(get_stored_value::<u8>(ARG_DECIMALS)).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn total_supply() {
    runtime::ret(CLValue::from_t(get_stored_value::<U256>(ARG_TOTAL_SUPPLY)).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn balance_of() {
    let address: Key = runtime::get_named_arg(ARG_ADDRESS);
    let balances_uref = get_balances_uref();
    let balance = read_balance_from(balances_uref, address);
    runtime::ret(CLValue::from_t(balance).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn allowance() {
    let spender: Key = runtime::get_named_arg(ARG_SPENDER);
    let owner: Key = runtime::get_named_arg(ARG_OWNER);
    let allowances_uref = get_allowances_uref();
    let val: U256 = read_allowance_from(allowances_uref, owner, spender);
    runtime::ret(CLValue::from_t(val).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn approve() {
    let owner = get_immediate_caller_address().unwrap_or_revert();
    let spender: Key = runtime::get_named_arg(ARG_SPENDER);
    if spender == owner {
        revert(Cep18Error::CannotTargetSelfUser);
    }
    let amount: U256 = runtime::get_named_arg(ARG_AMOUNT);
    let allowances_uref = get_allowances_uref();
    write_allowance_to(allowances_uref, owner, spender, amount);
    record_event_dictionary(Event::SetAllowance(SetAllowance {
        owner,
        spender,
        allowance: amount,
    }))
}

#[no_mangle]
pub extern "C" fn decrease_allowance() {
    let owner = get_immediate_caller_address().unwrap_or_revert();
    let spender: Key = runtime::get_named_arg(ARG_SPENDER);
    if spender == owner {
        revert(Cep18Error::CannotTargetSelfUser);
    }
    let amount: U256 = runtime::get_named_arg(ARG_AMOUNT);
    let allowances_uref = get_allowances_uref();
    let current_allowance = read_allowance_from(allowances_uref, owner, spender);
    let new_allowance = current_allowance.saturating_sub(amount);
    write_allowance_to(allowances_uref, owner, spender, new_allowance);
    record_event_dictionary(Event::DecreaseAllowance(DecreaseAllowance {
        owner,
        spender,
        decr_by: amount,
        allowance: new_allowance,
    }))
}

#[no_mangle]
pub extern "C" fn increase_allowance() {
    let owner = get_immediate_caller_address().unwrap_or_revert();
    let spender: Key = runtime::get_named_arg(ARG_SPENDER);
    if spender == owner {
        revert(Cep18Error::CannotTargetSelfUser);
    }
    let amount: U256 = runtime::get_named_arg(ARG_AMOUNT);
    let allowances_uref = get_allowances_uref();
    let current_allowance = read_allowance_from(allowances_uref, owner, spender);
    let new_allowance = current_allowance.saturating_add(amount);
    write_allowance_to(allowances_uref, owner, spender, new_allowance);
    record_event_dictionary(Event::IncreaseAllowance(IncreaseAllowance {
        owner,
        spender,
        allowance: new_allowance,
        inc_by: amount,
    }))
}

#[no_mangle]
pub extern "C" fn transfer() {
    let sender = get_immediate_caller_address().unwrap_or_revert();
    let recipient: Key = runtime::get_named_arg(ARG_RECIPIENT);
    if sender == recipient {
        revert(Cep18Error::CannotTargetSelfUser);
    }
    let amount: U256 = runtime::get_named_arg(ARG_AMOUNT);

    /* COWL */
    let data: Option<Bytes> =
        get_optional_named_arg_with_user_errors(ARG_DATA, Cep18Error::InvalidData);
    before_token_transfer(&sender, &sender, &recipient, amount, data);
    /*  */

    transfer_balance(sender, recipient, amount).unwrap_or_revert();
    record_event_dictionary(Event::Transfer(Transfer {
        sender,
        recipient,
        amount,
    }))
}

#[no_mangle]
pub extern "C" fn transfer_from() {
    let spender = get_immediate_caller_address().unwrap_or_revert();
    let recipient: Key = runtime::get_named_arg(ARG_RECIPIENT);
    let owner: Key = runtime::get_named_arg(ARG_OWNER);
    if owner == recipient {
        revert(Cep18Error::CannotTargetSelfUser);
    }
    let amount: U256 = runtime::get_named_arg(ARG_AMOUNT);
    if amount.is_zero() {
        return;
    }

    let allowances_uref = get_allowances_uref();
    let spender_allowance: U256 = read_allowance_from(allowances_uref, owner, spender);
    let new_spender_allowance = spender_allowance
        .checked_sub(amount)
        .ok_or(Cep18Error::InsufficientAllowance)
        .unwrap_or_revert();

    /* COWL */
    let data: Option<Bytes> =
        get_optional_named_arg_with_user_errors(ARG_DATA, Cep18Error::InvalidData);
    before_token_transfer(&spender, &owner, &recipient, amount, data);
    /*  */

    transfer_balance(owner, recipient, amount).unwrap_or_revert();
    write_allowance_to(allowances_uref, owner, spender, new_spender_allowance);
    record_event_dictionary(Event::TransferFrom(TransferFrom {
        spender,
        owner,
        recipient,
        amount,
    }))
}

#[no_mangle]
pub extern "C" fn mint() {
    if 0 == get_stored_value::<u8>(ARG_ENABLE_MINT_BURN) {
        revert(Cep18Error::MintBurnDisabled);
    }

    sec_check(vec![SecurityBadge::Admin, SecurityBadge::Minter]);

    let owner: Key = runtime::get_named_arg(ARG_OWNER);
    let amount: U256 = runtime::get_named_arg(ARG_AMOUNT);

    let balances_uref = get_balances_uref();
    let total_supply_uref = get_total_supply_uref();
    let new_balance = {
        let balance = read_balance_from(balances_uref, owner);
        balance
            .checked_add(amount)
            .ok_or(Cep18Error::Overflow)
            .unwrap_or_revert()
    };
    let new_total_supply = {
        let total_supply: U256 = read_total_supply_from(total_supply_uref);
        total_supply
            .checked_add(amount)
            .ok_or(Cep18Error::Overflow)
            .unwrap_or_revert()
    };
    write_balance_to(balances_uref, owner, new_balance);
    write_total_supply_to(total_supply_uref, new_total_supply);
    record_event_dictionary(Event::Mint(Mint {
        recipient: owner,
        amount,
    }))
}

#[no_mangle]
pub extern "C" fn burn() {
    if 0 == get_stored_value::<u8>(ARG_ENABLE_MINT_BURN) {
        revert(Cep18Error::MintBurnDisabled);
    }

    let owner: Key = runtime::get_named_arg(ARG_OWNER);

    if owner != get_immediate_caller_address().unwrap_or_revert() {
        revert(Cep18Error::InvalidBurnTarget);
    }

    let amount: U256 = runtime::get_named_arg(ARG_AMOUNT);
    let balances_uref = get_balances_uref();
    let total_supply_uref = get_total_supply_uref();
    let new_balance = {
        let balance = read_balance_from(balances_uref, owner);
        balance
            .checked_sub(amount)
            .ok_or(Cep18Error::InsufficientBalance)
            .unwrap_or_revert()
    };
    let new_total_supply = {
        let total_supply = read_total_supply_from(total_supply_uref);
        total_supply
            .checked_sub(amount)
            .ok_or(Cep18Error::Overflow)
            .unwrap_or_revert()
    };
    write_balance_to(balances_uref, owner, new_balance);
    write_total_supply_to(total_supply_uref, new_total_supply);
    record_event_dictionary(Event::Burn(Burn { owner, amount }))
}

/// Initiates the contracts states. Only used by the installer call,
/// later calls will cause it to revert.
#[no_mangle]
pub extern "C" fn init() {
    if get_key(DICT_ALLOWANCES).is_some() {
        revert(Cep18Error::AlreadyInitialized);
    }
    let package_hash = get_named_arg::<Key>(ARG_PACKAGE_HASH);
    put_key(ARG_PACKAGE_HASH, package_hash);
    storage::new_dictionary(DICT_ALLOWANCES).unwrap_or_revert();
    let balances_uref = storage::new_dictionary(DICT_BALANCES).unwrap_or_revert();
    let initial_supply = runtime::get_named_arg(ARG_TOTAL_SUPPLY);
    let caller = get_caller();
    write_balance_to(balances_uref, caller.into(), initial_supply);

    let security_badges_dict = storage::new_dictionary(DICT_SECURITY_BADGES).unwrap_or_revert();
    dictionary_put(
        security_badges_dict,
        &base64_encode(Key::from(get_caller()).to_bytes().unwrap_or_revert()),
        SecurityBadge::Admin,
    );

    let admin_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(ADMIN_LIST, Cep18Error::InvalidAdminList);
    let minter_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(MINTER_LIST, Cep18Error::InvalidMinterList);

    init_events();

    if let Some(minter_list) = minter_list {
        for minter in minter_list {
            dictionary_put(
                security_badges_dict,
                &base64_encode(minter.to_bytes().unwrap_or_revert()),
                SecurityBadge::Minter,
            );
        }
    }
    if let Some(admin_list) = admin_list {
        for admin in admin_list {
            dictionary_put(
                security_badges_dict,
                &base64_encode(admin.to_bytes().unwrap_or_revert()),
                SecurityBadge::Admin,
            );
        }
    }
    record_event_dictionary(Event::Mint(Mint {
        recipient: caller.into(),
        amount: initial_supply,
    }));

    /* COWL */
    let transfer_filter_contract_package_key: Option<Key> =
        get_optional_named_arg_with_user_errors::<Option<Key>>(
            ARG_TRANSFER_FILTER_CONTRACT_PACKAGE,
            Cep18Error::InvalidTransferFilterContract,
        )
        .unwrap_or_default();

    let transfer_filter_contract_package_hash: Option<ContractPackageHash> =
        transfer_filter_contract_package_key.map(|transfer_filter_contract_package_key| {
            ContractPackageHash::from(
                transfer_filter_contract_package_key
                    .into_hash()
                    .unwrap_or_revert(),
            )
        });

    runtime::put_key(
        ARG_TRANSFER_FILTER_CONTRACT_PACKAGE,
        storage::new_uref(transfer_filter_contract_package_hash).into(),
    );

    let transfer_filter_method: Option<String> =
        get_optional_named_arg_with_user_errors::<Option<String>>(
            ARG_TRANSFER_FILTER_METHOD,
            Cep18Error::InvalidTransferFilterMethod,
        )
        .unwrap_or_default();

    runtime::put_key(
        ARG_TRANSFER_FILTER_METHOD,
        storage::new_uref(transfer_filter_method).into(),
    );
    /*  */
}

/// Admin EntryPoint to manipulate the security access granted to users.
/// One user can only possess one access group badge.
/// Change strength: None > Admin > Minter
/// Change strength meaning by example: If user is added to both Minter and Admin they will be an
/// Admin, also if a user is added to Admin and None then they will be removed from having rights.
/// Beware: do not remove the last Admin because that will lock out all admin functionality.
#[no_mangle]
pub extern "C" fn change_security() {
    if 0 == get_stored_value::<u8>(ARG_ENABLE_MINT_BURN) {
        revert(Cep18Error::MintBurnDisabled);
    }
    sec_check(vec![SecurityBadge::Admin]);
    let admin_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(ADMIN_LIST, Cep18Error::InvalidAdminList);
    let minter_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(MINTER_LIST, Cep18Error::InvalidMinterList);
    let none_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(NONE_LIST, Cep18Error::InvalidNoneList);

    let mut badge_map: BTreeMap<Key, SecurityBadge> = BTreeMap::new();
    if let Some(minter_list) = minter_list {
        for account_key in minter_list {
            badge_map.insert(account_key, SecurityBadge::Minter);
        }
    }
    if let Some(admin_list) = admin_list {
        for account_key in admin_list {
            badge_map.insert(account_key, SecurityBadge::Admin);
        }
    }
    if let Some(none_list) = none_list {
        for account_key in none_list {
            badge_map.insert(account_key, SecurityBadge::None);
        }
    }

    let caller = get_immediate_caller_address().unwrap_or_revert();
    badge_map.remove(&caller);

    change_sec_badge(&badge_map);
    record_event_dictionary(Event::ChangeSecurity(ChangeSecurity {
        admin: get_immediate_caller_address().unwrap_or_revert(),
        sec_change_map: badge_map,
    }));
}

pub fn upgrade_contract(name: &str) {
    let entry_points = generate_entry_points();

    let contract_package_hash = runtime::get_key(&format!(
        "{PREFIX_CEP18}_{PREFIX_CONTRACT_PACKAGE_NAME}_{name}"
    ))
    .unwrap_or_revert()
    .into_hash()
    .map(ContractPackageHash::new)
    .unwrap_or_revert_with(Cep18Error::MissingPackageHashForUpgrade);

    let previous_contract_hash =
        runtime::get_key(&format!("{PREFIX_CEP18}_{PREFIX_CONTRACT_NAME}_{name}"))
            .unwrap_or_revert()
            .into_hash()
            .map(ContractHash::new)
            .unwrap_or_revert_with(Cep18Error::MissingPackageHashForUpgrade);

    let (contract_hash, contract_version) =
        storage::add_contract_version(contract_package_hash, entry_points, NamedKeys::new());

    storage::disable_contract_version(contract_package_hash, previous_contract_hash)
        .unwrap_or_revert();
    runtime::put_key(
        &format!("{PREFIX_CEP18}_{PREFIX_CONTRACT_NAME}_{name}"),
        contract_hash.into(),
    );
    runtime::put_key(
        &format!("{PREFIX_CEP18}_{PREFIX_CONTRACT_VERSION}_{name}"),
        storage::new_uref(contract_version).into(),
    );
    /* COWL */
    runtime::call_contract::<()>(contract_hash, ENTRY_POINT_UPGRADE, runtime_args! {});
    /*  */
}

/* COWL */
#[no_mangle]
pub extern "C" fn upgrade() {
    record_event_dictionary(Event::Upgrade(Upgrade {}));
}
/*  */

pub fn install_contract(name: &str) {
    let symbol: String = runtime::get_named_arg(ARG_SYMBOL);
    let decimals: u8 = runtime::get_named_arg(ARG_DECIMALS);
    let total_supply: U256 = runtime::get_named_arg(ARG_TOTAL_SUPPLY);
    let events_mode: u8 =
        get_optional_named_arg_with_user_errors(ARG_EVENTS_MODE, Cep18Error::InvalidEventsMode)
            .unwrap_or(0u8);

    let admin_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(ADMIN_LIST, Cep18Error::InvalidAdminList);
    let minter_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(MINTER_LIST, Cep18Error::InvalidMinterList);

    let enable_mint_burn: u8 = get_optional_named_arg_with_user_errors(
        ARG_ENABLE_MINT_BURN,
        Cep18Error::InvalidEnableMBFlag,
    )
    .unwrap_or(0);

    let mut named_keys = NamedKeys::new();
    named_keys.insert(ARG_NAME.to_string(), storage::new_uref(name).into());
    named_keys.insert(ARG_SYMBOL.to_string(), storage::new_uref(symbol).into());
    named_keys.insert(ARG_DECIMALS.to_string(), storage::new_uref(decimals).into());
    named_keys.insert(
        ARG_TOTAL_SUPPLY.to_string(),
        storage::new_uref(total_supply).into(),
    );
    named_keys.insert(
        ARG_EVENTS_MODE.to_string(),
        storage::new_uref(events_mode).into(),
    );
    named_keys.insert(
        ARG_ENABLE_MINT_BURN.to_string(),
        storage::new_uref(enable_mint_burn).into(),
    );
    let entry_points = generate_entry_points();

    let package_hash_name = format!("{PREFIX_CEP18}_{PREFIX_CONTRACT_PACKAGE_NAME}_{name}");

    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(package_hash_name.clone()),
        Some(format!("{PREFIX_CEP18}_{PREFIX_ACCESS_KEY_NAME}_{name}")),
    );
    let package_hash = runtime::get_key(&package_hash_name).unwrap_or_revert();

    // Store contract_hash and contract_version under the keys CONTRACT_NAME and CONTRACT_VERSION
    runtime::put_key(
        &format!("{PREFIX_CEP18}_{PREFIX_CONTRACT_NAME}_{name}"),
        contract_hash.into(),
    );
    runtime::put_key(
        &format!("{PREFIX_CEP18}_{PREFIX_CONTRACT_VERSION}_{name}"),
        storage::new_uref(contract_version).into(),
    );

    /* COWL */
    let transfer_filter_contract_package_key: Option<Key> = get_optional_named_arg_with_user_errors(
        ARG_TRANSFER_FILTER_CONTRACT_PACKAGE,
        Cep18Error::InvalidTransferFilterContract,
    );

    let transfer_filter_method: Option<String> = get_optional_named_arg_with_user_errors(
        ARG_TRANSFER_FILTER_METHOD,
        Cep18Error::InvalidTransferFilterMethod,
    );

    if let Some(_contract_key) = transfer_filter_contract_package_key {
        if transfer_filter_method.is_none() || transfer_filter_method.as_ref().unwrap().is_empty() {
            revert(Cep18Error::InvalidTransferFilterMethod);
        }
    }

    // Call contract to initialize it
    let mut init_args = runtime_args! {
        ARG_TOTAL_SUPPLY => total_supply,
        ARG_PACKAGE_HASH => package_hash,
        /* COWL */
        ARG_TRANSFER_FILTER_CONTRACT_PACKAGE => transfer_filter_contract_package_key,
        ARG_TRANSFER_FILTER_METHOD => transfer_filter_method,
        /*  */
    };

    if let Some(admin_list) = admin_list {
        init_args.insert(ADMIN_LIST, admin_list).unwrap_or_revert();
    }
    if let Some(minter_list) = minter_list {
        init_args
            .insert(MINTER_LIST, minter_list)
            .unwrap_or_revert();
    }

    runtime::call_contract::<()>(contract_hash, ENTRY_POINT_INIT, init_args);
}

#[no_mangle]
pub extern "C" fn call() {
    let name: String = runtime::get_named_arg(ARG_NAME);
    match runtime::get_key(&format!("{PREFIX_CEP18}_{PREFIX_ACCESS_KEY_NAME}_{name}")) {
        Some(_) => {
            upgrade_contract(&name);
        }
        None => {
            install_contract(&name);
        }
    }
}

/* COWL */
fn before_token_transfer(operator: &Key, from: &Key, to: &Key, amount: U256, data: Option<Bytes>) {
    if let Some(filter_contract_package) = get_transfer_filter_contract_package_hash() {
        if let Some(filter_method) = get_transfer_filter_method() {
            if amount == U256::zero() {
                runtime::revert(Cep18Error::InvalidAmount);
            }
            let mut args = RuntimeArgs::new();
            args.insert(ARG_OPERATOR, *operator)
                .unwrap_or_revert_with(Cep18Error::FailedToCreateArg);
            args.insert(ARG_FROM, *from)
                .unwrap_or_revert_with(Cep18Error::FailedToCreateArg);
            args.insert(ARG_TO, *to)
                .unwrap_or_revert_with(Cep18Error::FailedToCreateArg);
            args.insert(ARG_AMOUNT, amount)
                .unwrap_or_revert_with(Cep18Error::FailedToCreateArg);
            args.insert(ARG_DATA, data)
                .unwrap_or_revert_with(Cep18Error::FailedToCreateArg);

            let result: TransferFilterContractResult =
                call_versioned_contract::<u8>(filter_contract_package, None, &filter_method, args)
                    .into();

            if TransferFilterContractResult::DenyTransfer == result {
                revert(Cep18Error::TransferFilterContractDenied);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn set_transfer_filter() {
    // Only the installing account can change the mutable variables.
    sec_check(vec![SecurityBadge::Admin]);

    let caller = get_immediate_caller_address().unwrap_or_revert();

    let maybe_transfer_filter_contract_package_key: Option<Key> =
        get_named_arg(ARG_TRANSFER_FILTER_CONTRACT_PACKAGE);

    let maybe_transfer_filter_method: Option<String> = get_named_arg(ARG_TRANSFER_FILTER_METHOD);

    let maybe_transfer_filter_contract_package_hash: Option<ContractPackageHash> =
        maybe_transfer_filter_contract_package_key.map(|transfer_filter_contract_package_key| {
            ContractPackageHash::from(
                transfer_filter_contract_package_key
                    .into_hash()
                    .unwrap_or_revert(),
            )
        });

    runtime::put_key(
        ARG_TRANSFER_FILTER_CONTRACT_PACKAGE,
        storage::new_uref(maybe_transfer_filter_contract_package_hash).into(),
    );

    if maybe_transfer_filter_contract_package_key.is_some()
        && maybe_transfer_filter_method.is_some()
        && maybe_transfer_filter_method.as_ref().unwrap().is_empty()
    {
        revert(Cep18Error::InvalidTransferFilterMethod);
    }

    runtime::put_key(
        ARG_TRANSFER_FILTER_METHOD,
        storage::new_uref(maybe_transfer_filter_method.clone()).into(),
    );

    record_event_dictionary(Event::TransferFilterUpdate(TransferFilterUpdate {
        key: caller,
        transfer_filter_contract_package_key: maybe_transfer_filter_contract_package_key,
        transfer_filter_method: maybe_transfer_filter_method,
    }));
}
/*  */
