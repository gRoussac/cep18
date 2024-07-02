use core::convert::TryFrom;

use alloc::{collections::BTreeMap, format, string::String, vec::Vec};
use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{Key, U256};

use crate::{
    constants::{EVENTS, EVENTS_MODE},
    modalities::EventsMode,
    utils::{read_from, SecurityBadge},
    Cep18Error,
};

use casper_event_standard::{emit, Event, Schemas};

pub fn record_event_dictionary(event: Event) {
    let events_mode: EventsMode = EventsMode::try_from(read_from::<u8>(EVENTS_MODE))
        .unwrap_or_revert_with(Cep18Error::InvalidEventsMode);

    match events_mode {
        EventsMode::NoEvents => {}
        EventsMode::CES => ces(event),
        EventsMode::Native => runtime::emit_message(EVENTS, &format!("{event:?}").into())
            .unwrap_or_revert_with(Cep18Error::FailedToWriteMessage),
        EventsMode::NativeNCES => {
            runtime::emit_message(EVENTS, &format!("{event:?}").into())
                .unwrap_or_revert_with(Cep18Error::FailedToWriteMessage);
            ces(event);
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Mint(Mint),
    Burn(Burn),
    SetAllowance(SetAllowance),
    IncreaseAllowance(IncreaseAllowance),
    DecreaseAllowance(DecreaseAllowance),
    Transfer(Transfer),
    TransferFrom(TransferFrom),
    ChangeSecurity(ChangeSecurity),
    BalanceMigration(BalanceMigration),
    AllowanceMigration(AllowanceMigration),
    ChangeEventsMode(ChangeEventsMode)
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Mint {
    pub recipient: Key,
    pub amount: U256,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Burn {
    pub owner: Key,
    pub amount: U256,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct SetAllowance {
    pub owner: Key,
    pub spender: Key,
    pub allowance: U256,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct IncreaseAllowance {
    pub owner: Key,
    pub spender: Key,
    pub allowance: U256,
    pub inc_by: U256,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct DecreaseAllowance {
    pub owner: Key,
    pub spender: Key,
    pub allowance: U256,
    pub decr_by: U256,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Transfer {
    pub sender: Key,
    pub recipient: Key,
    pub amount: U256,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct TransferFrom {
    pub spender: Key,
    pub owner: Key,
    pub recipient: Key,
    pub amount: U256,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct ChangeSecurity {
    pub admin: Key,
    pub sec_change_map: BTreeMap<Key, SecurityBadge>,
}

/// `success_list` -> Vec<(Key,Key)> where the tuple is the pair of old_key and new_key.
/// `failure_map` -> BTreeMap<Key, String> where the key is the provided old_key, and the String
/// value is the failure reason, while the String is the failure reason.
#[derive(Event, Debug, PartialEq, Eq)]
pub struct BalanceMigration {
    pub success_map: Vec<(Key, Key)>,
    pub failure_map: BTreeMap<Key, String>,
}

/// `success_list` -> Vec<(Key,Key)> where one tuple is the pair of old_key and new_key for the
/// spender and another is the same for owner.
/// `failure_map` -> BTreeMap<(Key,Key), String> where the Key tuples is the pair of old_spender_key
/// and old_owner_key, while the String is the failure reason.
#[derive(Event, Debug, PartialEq, Eq)]
pub struct AllowanceMigration {
    pub success_map: Vec<((Key, Key), (Key, Key))>,
    pub failure_map: BTreeMap<(Key, Option<Key>), String>,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct ChangeEventsMode {
    pub events_mode: u8,
}

fn ces(event: Event) {
    match event {
        Event::Mint(ev) => emit(ev),
        Event::Burn(ev) => emit(ev),
        Event::SetAllowance(ev) => emit(ev),
        Event::IncreaseAllowance(ev) => emit(ev),
        Event::DecreaseAllowance(ev) => emit(ev),
        Event::Transfer(ev) => emit(ev),
        Event::TransferFrom(ev) => emit(ev),
        Event::ChangeSecurity(ev) => emit(ev),
        Event::BalanceMigration(ev) => emit(ev),
        Event::AllowanceMigration(ev) => emit(ev),
        Event::ChangeEventsMode(ev) => emit(ev),
    }
}

pub fn init_events() {
    let events_mode: EventsMode = EventsMode::try_from(read_from::<u8>(EVENTS_MODE))
        .unwrap_or_revert_with(Cep18Error::InvalidEventsMode);

    if [EventsMode::CES, EventsMode::NativeNCES].contains(&events_mode) {
        let schemas = Schemas::new()
            .with::<Mint>()
            .with::<Burn>()
            .with::<SetAllowance>()
            .with::<IncreaseAllowance>()
            .with::<DecreaseAllowance>()
            .with::<Transfer>()
            .with::<TransferFrom>()
            .with::<ChangeSecurity>()
            .with::<BalanceMigration>()
            .with::<AllowanceMigration>()
            .with::<ChangeEventsMode>();
        casper_event_standard::init(schemas);
    }
}
