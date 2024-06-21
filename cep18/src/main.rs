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
        runtime::{
            self, call_contract, get_caller, get_key, get_named_arg, manage_message_topic, put_key,
            revert,
        },
        storage::{self, dictionary_put, named_dictionary_get, named_dictionary_put},
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
    CHANGE_EVENTS_MODE_ENTRY_POINT_NAME, CONTRACT_HASH, CONTRACT_NAME_PREFIX,
    CONTRACT_VERSION_PREFIX, DECIMALS, ENABLE_MINT_BURN, EVENTS, EVENTS_MODE, HASH_KEY_NAME_PREFIX,
    INIT_ENTRY_POINT_NAME, MINTER_LIST, NAME, NONE_LIST, OWNER, PACKAGE_HASH, RECIPIENT, REVERT,
    SECURITY_BADGES, SPENDER, SYMBOL, TOTAL_SUPPLY, USER_KEY_MAP,
};
pub use error::Cep18Error;
use events::{
    init_events, AllowanceMigration, BalanceMigration, Burn, ChangeSecurity, DecreaseAllowance,
    Event, IncreaseAllowance, Mint, SetAllowance, Transfer, TransferFrom,
};
use modalities::EventsMode;
use utils::{
    get_immediate_caller_address, get_total_supply_uref, read_from, read_total_supply_from,
    sec_check, write_total_supply_to, SecurityBadge,
};

#[no_mangle]
pub extern "C" fn name() {
    runtime::ret(CLValue::from_t(utils::read_from::<String>(NAME)).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn symbol() {
    runtime::ret(CLValue::from_t(utils::read_from::<String>(SYMBOL)).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn decimals() {
    runtime::ret(CLValue::from_t(utils::read_from::<u8>(DECIMALS)).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn total_supply() {
    runtime::ret(CLValue::from_t(utils::read_from::<U256>(TOTAL_SUPPLY)).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn balance_of() {
    let address: Key = runtime::get_named_arg(ADDRESS);
    let balances_uref = get_balances_uref();
    let balance = balances::read_balance_from(balances_uref, address);
    runtime::ret(CLValue::from_t(balance).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn allowance() {
    let spender: Key = runtime::get_named_arg(SPENDER);
    let owner: Key = runtime::get_named_arg(OWNER);
    let allowances_uref = get_allowances_uref();
    let val: U256 = read_allowance_from(allowances_uref, owner, spender);
    runtime::ret(CLValue::from_t(val).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn approve() {
    let owner = utils::get_immediate_caller_address().unwrap_or_revert();
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
    let owner = utils::get_immediate_caller_address().unwrap_or_revert();
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
    let owner = utils::get_immediate_caller_address().unwrap_or_revert();
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
    let sender = utils::get_immediate_caller_address().unwrap_or_revert();
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
    let spender = utils::get_immediate_caller_address().unwrap_or_revert();
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
        .ok_or(Cep18Error::InsufficientAllowance)
        .unwrap_or_revert();

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
    events::record_event_dictionary(Event::Mint(Mint {
        recipient: owner,
        amount,
    }))
}

#[no_mangle]
pub extern "C" fn burn() {
    if 0 == read_from::<u8>(ENABLE_MINT_BURN) {
        revert(Cep18Error::MintBurnDisabled);
    }

    let owner: Key = runtime::get_named_arg(OWNER);

    if owner != get_immediate_caller_address().unwrap_or_revert() {
        revert(Cep18Error::InvalidBurnTarget);
    }

    let amount: U256 = runtime::get_named_arg(AMOUNT);
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

    storage::new_dictionary(ALLOWANCES).unwrap_or_revert();
    let balances_uref = storage::new_dictionary(BALANCES).unwrap_or_revert();
    let initial_supply = runtime::get_named_arg(TOTAL_SUPPLY);
    let caller = get_caller();
    write_balance_to(
        balances_uref,
        Key::AddressableEntity(EntityAddr::Account(caller.value())),
        initial_supply,
    );

    let security_badges_dict = storage::new_dictionary(SECURITY_BADGES).unwrap_or_revert();
    dictionary_put(
        security_badges_dict,
        &base64::encode(
            Key::AddressableEntity(EntityAddr::Account(caller.value()))
                .to_bytes()
                .unwrap_or_revert(),
        ),
        SecurityBadge::Admin,
    );

    let admin_list: Option<Vec<Key>> =
        utils::get_optional_named_arg_with_user_errors(ADMIN_LIST, Cep18Error::InvalidAdminList);
    let minter_list: Option<Vec<Key>> =
        utils::get_optional_named_arg_with_user_errors(MINTER_LIST, Cep18Error::InvalidMinterList);

    let events_mode: EventsMode =
        EventsMode::try_from(read_from::<u8>(EVENTS_MODE)).unwrap_or_revert();

    if [EventsMode::CES, EventsMode::NativeNCES].contains(&events_mode) {
        init_events();
    }

    if let Some(minter_list) = minter_list {
        for minter in minter_list {
            dictionary_put(
                security_badges_dict,
                &base64::encode(minter.to_bytes().unwrap_or_revert()),
                SecurityBadge::Minter,
            );
        }
    }
    if let Some(admin_list) = admin_list {
        for admin in admin_list {
            dictionary_put(
                security_badges_dict,
                &base64::encode(admin.to_bytes().unwrap_or_revert()),
                SecurityBadge::Admin,
            );
        }
    }
    events::record_event_dictionary(Event::Mint(Mint {
        recipient: Key::AddressableEntity(EntityAddr::Account(caller.value())),
        amount: initial_supply,
    }))
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
        utils::get_optional_named_arg_with_user_errors(ADMIN_LIST, Cep18Error::InvalidAdminList);
    let minter_list: Option<Vec<Key>> =
        utils::get_optional_named_arg_with_user_errors(MINTER_LIST, Cep18Error::InvalidMinterList);
    let none_list: Option<Vec<Key>> =
        utils::get_optional_named_arg_with_user_errors(NONE_LIST, Cep18Error::InvalidNoneList);

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

    utils::change_sec_badge(&badge_map);
    events::record_event_dictionary(Event::ChangeSecurity(ChangeSecurity {
        admin: get_immediate_caller_address().unwrap_or_revert(),
        sec_change_map: badge_map,
    }));
}

/// Entrypoint to migrate user and contract keys regarding balances provided as argument from 1.x to
/// 2.x key/hash storage version. Argument is a single BTreeMap<Key, bool>. The key is the already
/// stored key in the system (previously Key::Hash or Key::Account), while the bool value is the
/// verification for the key's state of being an account or a contract (true for account, false for
/// contract). Going forward these will be stored are Key::AddressableEntity(EntityAddr::Account)
/// and Key::AddressableEntity(EntityAddr::SmartContract) respectively.
#[no_mangle]
pub fn migrate_user_balance_keys() {
    let event_on: bool = get_named_arg(EVENTS);
    let revert_on: bool = get_named_arg(REVERT);
    let mut success_map: Vec<(Key, Key)> = Vec::new();
    let mut failure_map: BTreeMap<Key, String> = BTreeMap::new();

    let keys: BTreeMap<Key, bool> = get_named_arg(USER_KEY_MAP);
    let balances_uref = get_balances_uref();
    for (old_key, is_account_flag) in keys {
        let migrated_key = match old_key {
            Key::Account(account_hash) => {
                if !is_account_flag {
                    if event_on {
                        failure_map.insert(old_key, String::from("FlagMismatch"));
                    } else if revert_on {
                        revert(Cep18Error::KeyTypeMigrationMismatch)
                    }
                    continue;
                }
                Key::AddressableEntity(EntityAddr::Account(account_hash.value()))
            }
            Key::Hash(contract_package) => {
                if is_account_flag {
                    if event_on {
                        failure_map.insert(old_key, String::from("FlagMismatch"));
                    } else if revert_on {
                        revert(Cep18Error::KeyTypeMigrationMismatch)
                    }
                    continue;
                }
                Key::AddressableEntity(EntityAddr::SmartContract(contract_package))
            }
            _ => {
                if event_on {
                    failure_map.insert(old_key, String::from("WrongKeyType"));
                } else if revert_on {
                    revert(Cep18Error::InvalidKeyType)
                }
                continue;
            }
        };
        let old_balance = read_balance_from(balances_uref, old_key);
        if old_balance > U256::zero() {
            let new_key_existing_balance = read_balance_from(balances_uref, migrated_key);
            write_balance_to(balances_uref, old_key, U256::zero());
            write_balance_to(
                balances_uref,
                migrated_key,
                new_key_existing_balance + old_balance,
            )
        } else if event_on {
            failure_map.insert(old_key, String::from("NoOldKeyBal"));
        } else if revert_on {
            revert(Cep18Error::InsufficientBalance)
        }
        success_map.push((old_key, migrated_key));
    }

    if event_on {
        events::record_event_dictionary(Event::BalanceMigration(BalanceMigration {
            success_map,
            failure_map,
        }));
    }
}

/// Entrypoint to migrate users' and contracts' keys regarding allowances provided as argument from
/// 1.x to 2.x key/hash storage version. Argument is a single BTreeMap<Key, Vec<Key>>. The key is
/// the already stored key in the system (previously Key::Hash or Key::Account), while the Vec<Key>
/// is a list of allowance keys, whose owners have already been migrated. Going forward these will
/// be stored are Key::AddressableEntity(EntityAddr::Account)
/// and Key::AddressableEntity(EntityAddr::SmartContract) respectively.
#[no_mangle]
pub fn migrate_user_allowance_keys() {
    let event_on: bool = get_named_arg(EVENTS);
    let revert_on: bool = get_named_arg(REVERT);
    let mut success_map: Vec<((Key, Key), (Key, Key))> = Vec::new();
    let mut failure_map: BTreeMap<(Key, Option<Key>), String> = BTreeMap::new();

    let keys: BTreeMap<(Key, bool), Vec<(Key, bool)>> = get_named_arg(USER_KEY_MAP);
    let allowances_uref = get_allowances_uref();
    for ((spender_key, spender_is_account_flag), allowance_owner_keys) in keys {
        let migrated_spender_key = match spender_key {
            Key::Account(account_hash) => {
                if !spender_is_account_flag {
                    if event_on {
                        failure_map
                            .insert((spender_key, None), String::from("SpenderFlagMismatch"));
                        continue;
                    } else if revert_on {
                        revert(Cep18Error::KeyTypeMigrationMismatch)
                    }
                }
                Key::AddressableEntity(EntityAddr::Account(account_hash.value()))
            }
            Key::Hash(contract_package) => {
                if spender_is_account_flag {
                    if event_on {
                        failure_map
                            .insert((spender_key, None), String::from("SpenderFlagMismatch"));
                        continue;
                    } else if revert_on {
                        revert(Cep18Error::KeyTypeMigrationMismatch)
                    }
                }
                Key::AddressableEntity(EntityAddr::SmartContract(contract_package))
            }
            _ => {
                if event_on {
                    failure_map.insert((spender_key, None), String::from("SpenderWrongKeyType"));
                } else if revert_on {
                    revert(Cep18Error::InvalidKeyType)
                }
                continue;
            }
        };
        for (owner_key, owner_is_account_flag) in allowance_owner_keys {
            let migrated_owner_key = match owner_key {
                Key::Account(account_hash) => {
                    if !owner_is_account_flag {
                        if event_on {
                            failure_map.insert(
                                (spender_key, Some(owner_key)),
                                String::from("OwnerFlagMismatch"),
                            );
                        } else if revert_on {
                            revert(Cep18Error::KeyTypeMigrationMismatch)
                        }
                        continue;
                    }
                    Key::AddressableEntity(EntityAddr::Account(account_hash.value()))
                }
                Key::Hash(contract_package) => {
                    if owner_is_account_flag {
                        if event_on {
                            failure_map.insert(
                                (spender_key, Some(owner_key)),
                                String::from("OwnerFlagMismatch"),
                            );
                            continue;
                        } else if revert_on {
                            revert(Cep18Error::KeyTypeMigrationMismatch)
                        }
                    }
                    Key::AddressableEntity(EntityAddr::SmartContract(contract_package))
                }
                _ => {
                    if event_on {
                        failure_map.insert(
                            (spender_key, Some(owner_key)),
                            String::from("OwnerWrongKeyType"),
                        );
                    } else if revert_on {
                        revert(Cep18Error::InvalidKeyType)
                    }
                    continue;
                }
            };
            let old_allowance =
                read_allowance_from(allowances_uref, migrated_owner_key, migrated_spender_key);
            if old_allowance > U256::zero() {
                let new_key_existing_allowance =
                    read_allowance_from(allowances_uref, migrated_owner_key, migrated_spender_key);
                write_allowance_to(
                    allowances_uref,
                    migrated_owner_key,
                    migrated_spender_key,
                    U256::zero(),
                );
                write_allowance_to(
                    allowances_uref,
                    migrated_owner_key,
                    migrated_spender_key,
                    new_key_existing_allowance + old_allowance,
                )
            } else if event_on {
                failure_map.insert(
                    (spender_key, Some(owner_key)),
                    String::from("NoOldKeyAllowance"),
                );
            } else if revert_on {
                revert(Cep18Error::InsufficientAllowance)
            }
            success_map.push((
                (spender_key, migrated_spender_key),
                (owner_key, migrated_owner_key),
            ));
        }
    }
    if event_on {
        events::record_event_dictionary(Event::AllowanceMigration(AllowanceMigration {
            success_map,
            failure_map,
        }));
    }
}

#[no_mangle]
pub fn migrate_sec_keys() {
    let event_on: bool = get_named_arg(EVENTS);
    let revert_on: bool = get_named_arg(REVERT);
    let mut success_map: Vec<(Key, Key)> = Vec::new();
    let mut failure_map: BTreeMap<Key, String> = BTreeMap::new();

    let keys: BTreeMap<Key, bool> = get_named_arg(USER_KEY_MAP);
    for (old_key, is_account_flag) in keys {
        let migrated_key = match old_key {
            Key::Account(account_hash) => {
                if !is_account_flag {
                    if event_on {
                        failure_map.insert(old_key, String::from("FlagMismatch"));
                    } else if revert_on {
                        revert(Cep18Error::KeyTypeMigrationMismatch)
                    }
                    continue;
                }
                Key::AddressableEntity(EntityAddr::Account(account_hash.value()))
            }
            Key::Hash(contract_package) => {
                if is_account_flag {
                    if event_on {
                        failure_map.insert(old_key, String::from("FlagMismatch"));
                    } else if revert_on {
                        revert(Cep18Error::KeyTypeMigrationMismatch)
                    }
                    continue;
                }
                Key::AddressableEntity(EntityAddr::SmartContract(contract_package))
            }
            _ => {
                if event_on {
                    failure_map.insert(old_key, String::from("WrongKeyType"));
                } else if revert_on {
                    revert(Cep18Error::InvalidKeyType)
                }
                continue;
            }
        };
        let old_user_sec_key = old_key.to_bytes().unwrap_or_revert();
        let old_encoded_user_sec_key = base64::encode(old_user_sec_key);

        let user_sec_key = migrated_key.to_bytes().unwrap_or_revert();
        let migrated_encoded_user_sec_key = base64::encode(user_sec_key);

        let sec: SecurityBadge = named_dictionary_get(SECURITY_BADGES, &old_encoded_user_sec_key)
            .unwrap_or_revert()
            .unwrap_or(SecurityBadge::None);
        if ![SecurityBadge::Admin, SecurityBadge::Minter].contains(&sec) {
            named_dictionary_put(SECURITY_BADGES, &migrated_encoded_user_sec_key, sec);
        } else if event_on {
            failure_map.insert(old_key, String::from("NoValidBadge"));
        } else if revert_on {
            revert(Cep18Error::InsufficientRights)
        }
        success_map.push((old_key, migrated_key));
    }

    if event_on {
        events::record_event_dictionary(Event::BalanceMigration(BalanceMigration {
            success_map,
            failure_map,
        }));
    }
}

#[no_mangle]
fn change_events_mode() {
    sec_check(vec![SecurityBadge::Admin]);
    let events_mode: EventsMode =
        EventsMode::try_from(read_from::<u8>(EVENTS_MODE)).unwrap_or_revert();

    match events_mode {
        EventsMode::NoEvents => {}
        EventsMode::CES => init_events(),
        EventsMode::Native => {
            manage_message_topic(EVENTS, MessageTopicOperation::Add).unwrap_or_revert()
        }
        EventsMode::NativeNCES => {
            init_events();
            manage_message_topic(EVENTS, MessageTopicOperation::Add).unwrap_or_revert()
        }
    }

    put_key(EVENTS_MODE, storage::new_uref(events_mode as u8).into());
}

pub fn upgrade(name: &str) {
    let entry_points = generate_entry_points();

    let old_contract_package_hash =
        match runtime::get_key(&format!("{HASH_KEY_NAME_PREFIX}{name}")).unwrap_or_revert() {
            Key::Hash(contract_hash) => contract_hash,
            Key::AddressableEntity(EntityAddr::SmartContract(contract_hash)) => contract_hash,
            Key::Package(package_hash) => package_hash,
            _ => revert(Cep18Error::MissingPackageHashForUpgrade),
        };
    let contract_package_hash = PackageHash::new(old_contract_package_hash);

    let previous_contract_hash =
        match runtime::get_key(&format!("{CONTRACT_NAME_PREFIX}{name}")).unwrap_or_revert() {
            Key::Hash(contract_hash) => contract_hash,
            Key::AddressableEntity(EntityAddr::SmartContract(contract_hash)) => contract_hash,
            _ => revert(Cep18Error::MissingContractHashForUpgrade),
        };
    let converted_previous_contract_hash = AddressableEntityHash::new(previous_contract_hash);

    let (contract_hash, contract_version) = storage::add_contract_version(
        contract_package_hash,
        entry_points,
        NamedKeys::new(),
        BTreeMap::new(),
    );

    storage::disable_contract_version(contract_package_hash, converted_previous_contract_hash)
        .unwrap_or_revert();

    let events_mode = utils::get_optional_named_arg_with_user_errors::<u8>(
        EVENTS_MODE,
        Cep18Error::InvalidEventsMode,
    );
    if let Some(events_mode) = events_mode {
        call_contract::<()>(
            contract_hash,
            CHANGE_EVENTS_MODE_ENTRY_POINT_NAME,
            runtime_args! {
                EVENTS_MODE => events_mode
            },
        );
    }

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
}

pub fn install_contract(name: &str) {
    let symbol: String = runtime::get_named_arg(SYMBOL);
    let decimals: u8 = runtime::get_named_arg(DECIMALS);
    let total_supply: U256 = runtime::get_named_arg(TOTAL_SUPPLY);
    let events_mode_arg: u8 =
        utils::get_optional_named_arg_with_user_errors(EVENTS_MODE, Cep18Error::InvalidEventsMode)
            .unwrap_or(0u8);

    let events_mode: EventsMode = EventsMode::try_from(events_mode_arg).unwrap_or_revert();

    let admin_list: Option<Vec<Key>> =
        utils::get_optional_named_arg_with_user_errors(ADMIN_LIST, Cep18Error::InvalidAdminList);
    let minter_list: Option<Vec<Key>> =
        utils::get_optional_named_arg_with_user_errors(MINTER_LIST, Cep18Error::InvalidMinterList);

    let enable_mint_burn: u8 = utils::get_optional_named_arg_with_user_errors(
        ENABLE_MINT_BURN,
        Cep18Error::InvalidEnableMBFlag,
    )
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
        storage::new_uref(events_mode_arg).into(),
    );
    named_keys.insert(
        ENABLE_MINT_BURN.to_string(),
        storage::new_uref(enable_mint_burn).into(),
    );
    let entry_points = generate_entry_points();

    let message_topics = if events_mode == EventsMode::Native {
        let mut message_topics = BTreeMap::new();
        message_topics.insert(EVENTS.to_string(), MessageTopicOperation::Add);
        Some(message_topics)
    } else {
        None
    };

    let hash_key_name = format!("{HASH_KEY_NAME_PREFIX}{name}");
    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(hash_key_name.clone()),
        Some(format!("{ACCESS_KEY_NAME_PREFIX}{name}")),
        message_topics,
    );
    let package_hash = runtime::get_key(&hash_key_name).unwrap_or_revert();

    // Store contract_hash and contract_version under the keys CONTRACT_NAME and CONTRACT_VERSION
    runtime::put_key(
        &format!("{CONTRACT_NAME_PREFIX}{name}"),
        Key::addressable_entity_key(EntityKindTag::SmartContract, contract_hash),
    );
    runtime::put_key(
        &format!("{CONTRACT_VERSION_PREFIX}{name}"),
        storage::new_uref(contract_version).into(),
    );
    // Call contract to initialize it
    let mut init_args = runtime_args! {TOTAL_SUPPLY => total_supply, PACKAGE_HASH => package_hash, CONTRACT_HASH => Key::addressable_entity_key(EntityKindTag::SmartContract, contract_hash)};

    if let Some(admin_list) = admin_list {
        init_args.insert(ADMIN_LIST, admin_list).unwrap_or_revert();
    }
    if let Some(minter_list) = minter_list {
        init_args
            .insert(MINTER_LIST, minter_list)
            .unwrap_or_revert();
    }

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
