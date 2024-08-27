#![no_std]
#![no_main]

extern crate alloc;

mod allowances;
mod balances;
pub mod constants;
pub mod entry_points;
mod error;
mod events;
mod modalities;
mod utils;

use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use allowances::{get_allowances_uref, read_allowance_from, write_allowance_to};
use balances::{get_balances_uref, read_balance_from, transfer_balance, write_balance_to};
use entry_points::generate_entry_points;

use casper_contract::{
    contract_api::{
        runtime::{self, call_contract, get_caller, get_key, get_named_arg, put_key, revert},
        storage::{self, dictionary_put},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    addressable_entity::{EntityKindTag, NamedKeys},
    bytesrepr::ToBytes,
    contract_messages::MessageTopicOperation,
    runtime_args, AddressableEntityHash, CLValue, EntityAddr, Key, PackageHash, U256,
};

use constants::{
    ACCESS_KEY_NAME_PREFIX, ADDRESS, ADMIN_LIST, ALLOWANCES, AMOUNT, BALANCES,
    CHANGE_EVENTS_MODE_ENTRY_POINT_NAME, CONDOR, CONTRACT_HASH, CONTRACT_NAME_PREFIX,
    CONTRACT_VERSION_PREFIX, DECIMALS, ENABLE_MINT_BURN, EVENTS, EVENTS_MODE, HASH_KEY_NAME_PREFIX,
    INIT_ENTRY_POINT_NAME, MINTER_LIST, NAME, NONE_LIST, OWNER, PACKAGE_HASH, RECIPIENT,
    SECURITY_BADGES, SPENDER, SYMBOL, TOTAL_SUPPLY,
};
pub use error::Cep18Error;
use events::{
    init_events, Burn, ChangeEventsMode, ChangeSecurity, DecreaseAllowance, Event,
    IncreaseAllowance, Mint, SetAllowance, Transfer, TransferFrom,
};
use modalities::EventsMode;
use utils::{
    get_immediate_caller_key, get_optional_named_arg_with_user_errors, get_total_supply_uref,
    read_from, read_total_supply_from, sec_check, write_total_supply_to, SecurityBadge,
};

#[no_mangle]
pub extern "C" fn condor() {
    runtime::ret(
        CLValue::from_t(utils::read_from::<String>(CONDOR))
            .unwrap_or_revert_with(Cep18Error::FailedToReturnEntryPointResult),
    );
}

#[no_mangle]
pub extern "C" fn name() {
    runtime::ret(
        CLValue::from_t(utils::read_from::<String>(NAME))
            .unwrap_or_revert_with(Cep18Error::FailedToReturnEntryPointResult),
    );
}

#[no_mangle]
pub extern "C" fn symbol() {
    runtime::ret(
        CLValue::from_t(utils::read_from::<String>(SYMBOL))
            .unwrap_or_revert_with(Cep18Error::FailedToReturnEntryPointResult),
    );
}

#[no_mangle]
pub extern "C" fn decimals() {
    runtime::ret(
        CLValue::from_t(utils::read_from::<u8>(DECIMALS))
            .unwrap_or_revert_with(Cep18Error::FailedToReturnEntryPointResult),
    );
}

#[no_mangle]
pub extern "C" fn total_supply() {
    runtime::ret(
        CLValue::from_t(utils::read_from::<U256>(TOTAL_SUPPLY))
            .unwrap_or_revert_with(Cep18Error::FailedToReturnEntryPointResult),
    );
}

#[no_mangle]
pub extern "C" fn balance_of() {
    let address: Key = runtime::get_named_arg(ADDRESS);
    let balances_uref = get_balances_uref();
    let balance = balances::read_balance_from(balances_uref, address);
    runtime::ret(
        CLValue::from_t(balance).unwrap_or_revert_with(Cep18Error::FailedToReturnEntryPointResult),
    );
}

#[no_mangle]
pub extern "C" fn allowance() {
    let spender: Key = runtime::get_named_arg(SPENDER);
    let owner: Key = runtime::get_named_arg(OWNER);
    let allowances_uref = get_allowances_uref();
    let val: U256 = read_allowance_from(allowances_uref, owner, spender);
    runtime::ret(
        CLValue::from_t(val).unwrap_or_revert_with(Cep18Error::FailedToReturnEntryPointResult),
    );
}

#[no_mangle]
pub extern "C" fn approve() {
    let owner = utils::get_immediate_caller_key()
        .unwrap_or_revert_with(Cep18Error::FailedToRetrieveImmediateCaller);
    let spender: Key = runtime::get_named_arg(SPENDER);
    if spender == owner {
        revert(Cep18Error::CannotTargetSelfUser);
    }
    let amount: U256 = runtime::get_named_arg(AMOUNT);
    let allowances_uref = get_allowances_uref();
    write_allowance_to(allowances_uref, owner, spender, amount);
    events::record_event_dictionary(Event::SetAllowance(SetAllowance {
        owner,
        spender,
        allowance: amount,
    }))
}

#[no_mangle]
pub extern "C" fn decrease_allowance() {
    let owner = utils::get_immediate_caller_key()
        .unwrap_or_revert_with(Cep18Error::FailedToRetrieveImmediateCaller);
    let spender: Key = runtime::get_named_arg(SPENDER);
    if spender == owner {
        revert(Cep18Error::CannotTargetSelfUser);
    }
    let amount: U256 = runtime::get_named_arg(AMOUNT);
    let allowances_uref = get_allowances_uref();
    let current_allowance = read_allowance_from(allowances_uref, owner, spender);
    let new_allowance = current_allowance.saturating_sub(amount);
    write_allowance_to(allowances_uref, owner, spender, new_allowance);
    events::record_event_dictionary(Event::DecreaseAllowance(DecreaseAllowance {
        owner,
        spender,
        decr_by: amount,
        allowance: new_allowance,
    }))
}

#[no_mangle]
pub extern "C" fn increase_allowance() {
    let owner = utils::get_immediate_caller_key()
        .unwrap_or_revert_with(Cep18Error::FailedToRetrieveImmediateCaller);
    let spender: Key = runtime::get_named_arg(SPENDER);
    if spender == owner {
        revert(Cep18Error::CannotTargetSelfUser);
    }
    let amount: U256 = runtime::get_named_arg(AMOUNT);
    let allowances_uref = get_allowances_uref();
    let current_allowance = read_allowance_from(allowances_uref, owner, spender);
    let new_allowance = current_allowance.saturating_add(amount);
    write_allowance_to(allowances_uref, owner, spender, new_allowance);
    events::record_event_dictionary(Event::IncreaseAllowance(IncreaseAllowance {
        owner,
        spender,
        allowance: new_allowance,
        inc_by: amount,
    }))
}

#[no_mangle]
pub extern "C" fn transfer() {
    let sender = utils::get_immediate_caller_key()
        .unwrap_or_revert_with(Cep18Error::FailedToRetrieveImmediateCaller);
    let recipient: Key = runtime::get_named_arg(RECIPIENT);
    if sender == recipient {
        revert(Cep18Error::CannotTargetSelfUser);
    }
    let amount: U256 = runtime::get_named_arg(AMOUNT);
    transfer_balance(sender, recipient, amount).unwrap_or_revert();
    events::record_event_dictionary(Event::Transfer(Transfer {
        sender,
        recipient,
        amount,
    }))
}

#[no_mangle]
pub extern "C" fn transfer_from() {
    let spender = utils::get_immediate_caller_key()
        .unwrap_or_revert_with(Cep18Error::FailedToRetrieveImmediateCaller);
    let recipient: Key = runtime::get_named_arg(RECIPIENT);
    let owner: Key = runtime::get_named_arg(OWNER);
    if owner == recipient {
        revert(Cep18Error::CannotTargetSelfUser);
    }
    let amount: U256 = runtime::get_named_arg(AMOUNT);
    if amount.is_zero() {
        return;
    }

    let allowances_uref = get_allowances_uref();
    let spender_allowance: U256 = read_allowance_from(allowances_uref, owner, spender);
    let new_spender_allowance = spender_allowance
        .checked_sub(amount)
        .unwrap_or_revert_with(Cep18Error::InsufficientAllowance);

    transfer_balance(owner, recipient, amount).unwrap_or_revert();
    write_allowance_to(allowances_uref, owner, spender, new_spender_allowance);
    events::record_event_dictionary(Event::TransferFrom(TransferFrom {
        spender,
        owner,
        recipient,
        amount,
    }))
}

#[no_mangle]
pub extern "C" fn mint() {
    if 0 == read_from::<u8>(ENABLE_MINT_BURN) {
        revert(Cep18Error::MintBurnDisabled);
    }

    sec_check(vec![SecurityBadge::Admin, SecurityBadge::Minter]);

    let owner: Key = runtime::get_named_arg(OWNER);
    let amount: U256 = runtime::get_named_arg(AMOUNT);

    let balances_uref = get_balances_uref();
    let total_supply_uref = get_total_supply_uref();
    let new_balance = {
        let balance = read_balance_from(balances_uref, owner);
        balance
            .checked_add(amount)
            .unwrap_or_revert_with(Cep18Error::Overflow)
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

    events::record_event_dictionary(Event::Mint(Mint {
        recipient: owner,
        amount,
    }));
}

#[no_mangle]
pub extern "C" fn burn() {
    if 0 == read_from::<u8>(ENABLE_MINT_BURN) {
        revert(Cep18Error::MintBurnDisabled);
    }

    let owner: Key = runtime::get_named_arg(OWNER);

    if owner
        != get_immediate_caller_key()
            .unwrap_or_revert_with(Cep18Error::FailedToRetrieveImmediateCaller)
    {
        revert(Cep18Error::InvalidBurnTarget);
    }

    let amount: U256 = runtime::get_named_arg(AMOUNT);
    let balances_uref = get_balances_uref();
    let total_supply_uref = get_total_supply_uref();
    let new_balance = {
        let balance = read_balance_from(balances_uref, owner);
        balance
            .checked_sub(amount)
            .unwrap_or_revert_with(Cep18Error::InsufficientBalance)
    };
    let new_total_supply = {
        let total_supply = read_total_supply_from(total_supply_uref);
        total_supply
            .checked_sub(amount)
            .ok_or(Cep18Error::Overflow)
            .unwrap_or_revert_with(Cep18Error::FailedToChangeTotalSupply)
    };
    write_balance_to(balances_uref, owner, new_balance);
    write_total_supply_to(total_supply_uref, new_total_supply);
    events::record_event_dictionary(Event::Burn(Burn { owner, amount }))
}

/// Initiates the contracts states. Only used by the installer call,
/// later calls will cause it to revert.
#[no_mangle]
pub extern "C" fn init() {
    if get_key(ALLOWANCES).is_some() {
        revert(Cep18Error::AlreadyInitialized);
    }
    let package_hash = get_named_arg::<Key>(PACKAGE_HASH);
    put_key(PACKAGE_HASH, package_hash);

    let contract_hash = get_named_arg::<Key>(CONTRACT_HASH);
    put_key(CONTRACT_HASH, contract_hash);

    storage::new_dictionary(ALLOWANCES).unwrap_or_revert_with(Cep18Error::FailedToCreateDictionary);
    let balances_uref = storage::new_dictionary(BALANCES)
        .unwrap_or_revert_with(Cep18Error::FailedToCreateDictionary);
    let initial_supply = runtime::get_named_arg(TOTAL_SUPPLY);
    let caller = get_caller();
    let initial_balance_holder_key = Key::Account(caller);

    write_balance_to(balances_uref, initial_balance_holder_key, initial_supply);

    let security_badges_dict = storage::new_dictionary(SECURITY_BADGES)
        .unwrap_or_revert_with(Cep18Error::FailedToCreateDictionary);
    dictionary_put(
        security_badges_dict,
        &base64::encode(
            initial_balance_holder_key
                .to_bytes()
                .unwrap_or_revert_with(Cep18Error::FailedToConvertBytes),
        ),
        SecurityBadge::Admin,
    );

    let admin_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(ADMIN_LIST, Cep18Error::InvalidAdminList);
    let minter_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(MINTER_LIST, Cep18Error::InvalidMinterList);

    let events_mode: EventsMode = EventsMode::try_from(get_named_arg::<u8>(EVENTS_MODE))
        .unwrap_or_revert_with(Cep18Error::InvalidEventsMode);

    if EventsMode::CES == events_mode {
        init_events();
    }

    if let Some(minter_list) = minter_list {
        for minter in minter_list {
            dictionary_put(
                security_badges_dict,
                &base64::encode(
                    minter
                        .to_bytes()
                        .unwrap_or_revert_with(Cep18Error::FailedToConvertBytes),
                ),
                SecurityBadge::Minter,
            );
        }
    }
    if let Some(admin_list) = admin_list {
        for admin in admin_list {
            dictionary_put(
                security_badges_dict,
                &base64::encode(
                    admin
                        .to_bytes()
                        .unwrap_or_revert_with(Cep18Error::FailedToConvertBytes),
                ),
                SecurityBadge::Admin,
            );
        }
    }

    events::record_event_dictionary(Event::Mint(Mint {
        recipient: initial_balance_holder_key,
        amount: initial_supply,
    }));
}

/// Admin EntryPoint to manipulate the security access granted to users.
/// One user can only possess one access group badge.
/// Change strength: None > Admin > Minter
/// Change strength meaning by example: If user is added to both Minter and Admin they will be an
/// Admin, also if a user is added to Admin and None then they will be removed from having rights.
/// Beware: do not remove the last Admin because that will lock out all admin functionality.
#[no_mangle]
pub extern "C" fn change_security() {
    if 0 == read_from::<u8>(ENABLE_MINT_BURN) {
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

    let caller = get_immediate_caller_key()
        .unwrap_or_revert_with(Cep18Error::FailedToRetrieveImmediateCaller);
    badge_map.remove(&caller);

    utils::change_sec_badge(&badge_map);
    events::record_event_dictionary(Event::ChangeSecurity(ChangeSecurity {
        admin: caller,
        sec_change_map: badge_map,
    }));
}

#[no_mangle]
fn change_events_mode() {
    sec_check(vec![SecurityBadge::Admin]);
    let events_mode: EventsMode = EventsMode::try_from(get_named_arg::<u8>(EVENTS_MODE))
        .unwrap_or_revert_with(Cep18Error::InvalidEventsMode);
    let old_events_mode: EventsMode = EventsMode::try_from(read_from::<u8>(EVENTS_MODE))
        .unwrap_or_revert_with(Cep18Error::InvalidEventsMode);
    if events_mode == old_events_mode {
        revert(Cep18Error::UnchangedEventsMode);
    }
    let events_mode_u8 = events_mode as u8;
    put_key(EVENTS_MODE, storage::new_uref(events_mode_u8).into());

    if get_key(casper_event_standard::EVENTS_DICT).is_none() {
        init_events()
    }
    events::record_event_dictionary(Event::ChangeEventsMode(ChangeEventsMode {
        events_mode: events_mode_u8,
    }));
}

pub fn upgrade(name: &str) {
    let entry_points = generate_entry_points();
    let old_contract_package_hash = match runtime::get_key(&format!("{HASH_KEY_NAME_PREFIX}{name}"))
        .unwrap_or_revert_with(Cep18Error::FailedToGetOldPackageKey)
    {
        Key::Hash(contract_hash) => contract_hash,
        Key::AddressableEntity(EntityAddr::SmartContract(contract_hash)) => contract_hash,
        Key::Package(package_hash) => package_hash,
        _ => revert(Cep18Error::MissingPackageHashForUpgrade),
    };
    let contract_package_hash = PackageHash::new(old_contract_package_hash);

    let previous_contract_hash = match runtime::get_key(&format!("{CONTRACT_NAME_PREFIX}{name}"))
        .unwrap_or_revert_with(Cep18Error::FailedToGetOldContractHashKey)
    {
        Key::Hash(contract_hash) => contract_hash,
        Key::AddressableEntity(EntityAddr::SmartContract(contract_hash)) => contract_hash,
        _ => revert(Cep18Error::MissingContractHashForUpgrade),
    };
    let converted_previous_contract_hash = AddressableEntityHash::new(previous_contract_hash);

    let events_mode =
        get_optional_named_arg_with_user_errors::<u8>(EVENTS_MODE, Cep18Error::InvalidEventsMode);

    let mut message_topics = BTreeMap::new();
    match get_key(CONDOR) {
        Some(_) => {}
        None => {
            message_topics.insert(EVENTS.to_string(), MessageTopicOperation::Add);
            put_key(CONDOR, storage::new_uref(CONDOR).into());
        }
    }

    let mut named_keys = NamedKeys::new();
    named_keys.insert(CONDOR.to_string(), storage::new_uref(CONDOR).into());

    let (contract_hash, contract_version) = storage::add_contract_version(
        contract_package_hash,
        entry_points,
        named_keys,
        message_topics,
    );

    storage::disable_contract_version(contract_package_hash, converted_previous_contract_hash)
        .unwrap_or_revert_with(Cep18Error::FailedToDisableContractVersion);

    // migrate old ContractPackageHash as PackageHash so it's stored in a uniform format with the
    // new `new_contract` implementation
    runtime::put_key(
        &format!("{HASH_KEY_NAME_PREFIX}{name}"),
        contract_package_hash.into(),
    );

    // ContractHash in previous versions, now AddressableEntityHash
    runtime::put_key(
        &format!("{CONTRACT_NAME_PREFIX}{name}"),
        Key::addressable_entity_key(EntityKindTag::SmartContract, contract_hash),
    );
    runtime::put_key(
        &format!("{CONTRACT_VERSION_PREFIX}{name}"),
        storage::new_uref(contract_version).into(),
    );

    if let Some(events_mode_u8) = events_mode {
        call_contract::<()>(
            contract_hash,
            CHANGE_EVENTS_MODE_ENTRY_POINT_NAME,
            runtime_args! {
                EVENTS_MODE => events_mode_u8,
            },
        );
    }
}

pub fn install_contract(name: &str) {
    let symbol: String = runtime::get_named_arg(SYMBOL);
    let decimals: u8 = runtime::get_named_arg(DECIMALS);
    let total_supply: U256 = runtime::get_named_arg(TOTAL_SUPPLY);
    let events_mode: u8 =
        get_optional_named_arg_with_user_errors(EVENTS_MODE, Cep18Error::InvalidEventsMode)
            .unwrap_or(0u8);

    let admin_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(ADMIN_LIST, Cep18Error::InvalidAdminList);
    let minter_list: Option<Vec<Key>> =
        get_optional_named_arg_with_user_errors(MINTER_LIST, Cep18Error::InvalidMinterList);

    let enable_mint_burn: u8 =
        get_optional_named_arg_with_user_errors(ENABLE_MINT_BURN, Cep18Error::InvalidEnableMBFlag)
            .unwrap_or(0);

    let mut named_keys = NamedKeys::new();
    named_keys.insert(NAME.to_string(), storage::new_uref(name).into());
    named_keys.insert(SYMBOL.to_string(), storage::new_uref(symbol).into());
    named_keys.insert(DECIMALS.to_string(), storage::new_uref(decimals).into());
    named_keys.insert(
        TOTAL_SUPPLY.to_string(),
        storage::new_uref(total_supply).into(),
    );
    named_keys.insert(
        EVENTS_MODE.to_string(),
        storage::new_uref(events_mode).into(),
    );
    named_keys.insert(
        ENABLE_MINT_BURN.to_string(),
        storage::new_uref(enable_mint_burn).into(),
    );

    let entry_points = generate_entry_points();

    let mut message_topics = BTreeMap::new();
    if [EventsMode::Native, EventsMode::NativeBytes]
        .contains(&events_mode.try_into().unwrap_or_default())
    {
        message_topics.insert(EVENTS.to_string(), MessageTopicOperation::Add);
    };

    let hash_key_name = format!("{HASH_KEY_NAME_PREFIX}{name}");
    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(hash_key_name.clone()),
        Some(format!("{ACCESS_KEY_NAME_PREFIX}{name}")),
        Some(message_topics),
    );
    let package_hash =
        runtime::get_key(&hash_key_name).unwrap_or_revert_with(Cep18Error::FailedToGetPackageKey);

    let contract_hash_key =
        Key::addressable_entity_key(EntityKindTag::SmartContract, contract_hash);

    // Store contract_hash and contract_version under the keys CONTRACT_NAME and CONTRACT_VERSION
    runtime::put_key(&format!("{CONTRACT_NAME_PREFIX}{name}"), contract_hash_key);
    runtime::put_key(
        &format!("{CONTRACT_VERSION_PREFIX}{name}"),
        storage::new_uref(contract_version).into(),
    );
    // Call contract to initialize it
    let mut init_args = runtime_args! {TOTAL_SUPPLY => total_supply, PACKAGE_HASH => package_hash, CONTRACT_HASH => contract_hash_key, EVENTS_MODE => events_mode};

    if let Some(admin_list) = admin_list {
        init_args
            .insert(ADMIN_LIST, admin_list)
            .unwrap_or_revert_with(Cep18Error::FailedToInsertToSecurityList);
    }
    if let Some(minter_list) = minter_list {
        init_args
            .insert(MINTER_LIST, minter_list)
            .unwrap_or_revert_with(Cep18Error::FailedToInsertToSecurityList);
    }

    put_key(CONDOR, storage::new_uref(CONDOR).into());
    runtime::call_contract::<()>(contract_hash, INIT_ENTRY_POINT_NAME, init_args);
}

#[no_mangle]
pub extern "C" fn call() {
    let name: String = runtime::get_named_arg(NAME);
    match runtime::get_key(&format!("{ACCESS_KEY_NAME_PREFIX}{name}")) {
        Some(_) => {
            upgrade(&name);
        }
        None => {
            install_contract(&name);
        }
    }
}
