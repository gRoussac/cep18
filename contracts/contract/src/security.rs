#[cfg(feature = "contract-support")]
use crate::{
    constants::DICT_SECURITY_BADGES,
    error::Cep18Error,
    utils::{base64_encode, get_immediate_caller_address, get_uref},
};
#[cfg(feature = "contract-support")]
use alloc::collections::BTreeMap;
use alloc::{vec, vec::Vec};
#[cfg(feature = "contract-support")]
use casper_contract::{
    contract_api::{
        runtime::revert,
        storage::{dictionary_get, dictionary_put},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
#[cfg(feature = "contract-support")]
use casper_types::Key;
use casper_types::{
    bytesrepr::{self, FromBytes, ToBytes},
    CLTyped,
};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SecurityBadge {
    Admin = 0,
    Minter = 1,
    None = 2,
}

impl CLTyped for SecurityBadge {
    fn cl_type() -> casper_types::CLType {
        casper_types::CLType::U8
    }
}

impl ToBytes for SecurityBadge {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        Ok(vec![*self as u8])
    }

    fn serialized_length(&self) -> usize {
        1
    }
}

impl FromBytes for SecurityBadge {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        Ok((
            match bytes[0] {
                0 => SecurityBadge::Admin,
                1 => SecurityBadge::Minter,
                2 => SecurityBadge::None,
                _ => return Err(bytesrepr::Error::LeftOverBytes),
            },
            &[],
        ))
    }
}

#[cfg(feature = "contract-support")]
pub fn sec_check(allowed_badge_list: Vec<SecurityBadge>) {
    let caller = get_immediate_caller_address()
        .unwrap_or_revert()
        .to_bytes()
        .unwrap_or_revert();
    if !allowed_badge_list.contains(
        &dictionary_get::<SecurityBadge>(get_uref(DICT_SECURITY_BADGES), &base64_encode(caller))
            .unwrap_or_revert()
            .unwrap_or_revert_with(Cep18Error::InsufficientRights),
    ) {
        revert(Cep18Error::InsufficientRights)
    }
}

#[cfg(feature = "contract-support")]
pub fn change_sec_badge(badge_map: &BTreeMap<Key, SecurityBadge>) {
    let sec_uref = get_uref(DICT_SECURITY_BADGES);
    for (&user, &badge) in badge_map {
        dictionary_put(
            sec_uref,
            &base64_encode(user.to_bytes().unwrap_or_revert()),
            badge,
        )
    }
}
