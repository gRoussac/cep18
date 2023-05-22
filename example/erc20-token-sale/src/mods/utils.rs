use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_erc20::constants::{
    BALANCES_KEY_NAME, ERC20_TOKEN_CONTRACT_KEY_NAME, OWNER_RUNTIME_ARG_NAME,
};
use casper_types::{account::AccountHash, ApiError, ContractHash, Key};

use super::{
    constants::{COUNT_INVESTMENTS_KEY, COUNT_INVESTORS_KEY, LEDGER, ONE},
    InvestingError,
};

fn _get_token_hash() -> ContractHash {
    {
        let token_key_hash = runtime::get_key(ERC20_TOKEN_CONTRACT_KEY_NAME)
            .unwrap_or_revert_with(InvestingError::MissingERC20TokenURef);
        if let Key::Hash(hash) = token_key_hash {
            ContractHash::new(hash)
        } else {
            runtime::revert(ApiError::User(66)); // TODO
        }
    }
}

pub fn _get_owner_hash() -> AccountHash {
    {
        let owner_key_hash = runtime::get_key(OWNER_RUNTIME_ARG_NAME)
            .unwrap_or_revert_with(InvestingError::MissingOwnerHash);
        if let Key::Hash(hash) = owner_key_hash {
            AccountHash::new(hash)
        } else {
            runtime::revert(ApiError::User(66)); // TODO
        }
    }
}

pub fn update_ledger_record(dictionary_item_key: &str) {
    // Acquiring the LEDGER seed URef to properly assign the dictionary item.
    let ledger_seed_uref = get_key_uref(LEDGER, InvestingError::MissingLedgerSeedURef.into());
    // This identifies an item within the dictionary and either creates or updates the associated value.
    match storage::dictionary_get::<u64>(ledger_seed_uref, dictionary_item_key).unwrap_or_revert() {
        None => {
            storage::dictionary_put(ledger_seed_uref, dictionary_item_key, ONE);
            // Update counter for investors
            counter_inc(
                COUNT_INVESTORS_KEY,
                InvestingError::MissingCountInvestorsKey,
            );
            counter_inc(
                COUNT_INVESTMENTS_KEY,
                InvestingError::MissingCountInvestmentsKey,
            );
        }
        Some(current_number_of_purchase) => {
            storage::dictionary_put(
                ledger_seed_uref,
                dictionary_item_key,
                current_number_of_purchase + ONE,
            );
            counter_inc(
                COUNT_INVESTMENTS_KEY,
                InvestingError::MissingCountInvestmentsKey,
            );
        }
    }
}

fn _update_balance(dictionary_item_key: &str, dictionary_item_value: u64) {
    // Acquiring the BALANCE seed URef to properly assign the dictionary item.
    let balance_seed_uref = get_key_uref(
        BALANCES_KEY_NAME,
        InvestingError::MissingBalancesSeedURef.into(),
    );
    match storage::dictionary_get::<u64>(balance_seed_uref, dictionary_item_key).unwrap_or_revert()
    {
        None => {
            storage::dictionary_put(
                balance_seed_uref,
                dictionary_item_key,
                dictionary_item_value,
            );
        }
        Some(current_token_balance) => {
            storage::dictionary_put(
                balance_seed_uref,
                dictionary_item_key,
                current_token_balance + dictionary_item_value,
            );
        }
    }
}

// fn make_dictionary_item_key(owner: Address) -> String {
//     let preimage = owner.to_bytes().unwrap_or_revert();
//     base64::encode(&preimage)
// }

pub fn get_key_uref(key: &str, error_key: ApiError) -> casper_types::URef {
    runtime::get_key(key)
        .unwrap_or_revert_with(error_key)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant)
}

fn counter_inc(key: &str, error_key: InvestingError) {
    storage::add(get_key_uref(key, error_key.into()), ONE);
}
