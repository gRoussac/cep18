use casper_engine_test_support::{
    ExecuteRequestBuilder, WasmTestBuilder, DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use casper_execution_engine::{
    core::{engine_state::Error as EngineStateError, execution},
    storage::global_state::in_memory::InMemoryGlobalState,
};
use casper_types::{
    account::AccountHash,
    runtime_args,
    system::{
        handle_payment::{ARG_AMOUNT, ARG_TARGET},
        mint::ARG_ID,
    },
    ApiError, PublicKey, RuntimeArgs, SecretKey,
};

pub fn assert_expected_error(actual_error: EngineStateError, error_code: u16, reason: &str) {
    let actual = format!("{actual_error:?}");
    let expected = format!(
        "{:?}",
        EngineStateError::Exec(execution::Error::Revert(ApiError::User(error_code)))
    );

    assert_eq!(
        actual, expected,
        "Error should match {error_code} with reason: {reason}"
    )
}

// Creates a dummy account and transfer funds to it
pub fn create_funded_dummy_account(
    builder: &mut WasmTestBuilder<InMemoryGlobalState>,
    account_string: Option<[u8; 32]>,
) -> AccountHash {
    let (_, account_public_key) = create_dummy_key_pair(account_string.unwrap_or([7u8; 32]));
    let account = account_public_key.to_account_hash();
    fund_account(builder, account);
    account
}

pub fn create_dummy_key_pair(account_string: [u8; 32]) -> (SecretKey, PublicKey) {
    let secret_key =
        SecretKey::ed25519_from_bytes(account_string).expect("failed to create secret key");
    let public_key = PublicKey::from(&secret_key);
    (secret_key, public_key)
}

pub fn fund_account(builder: &mut WasmTestBuilder<InMemoryGlobalState>, account: AccountHash) {
    let transfer = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_ADDR,
        runtime_args! {
            ARG_AMOUNT => DEFAULT_ACCOUNT_INITIAL_BALANCE / 10_u64,
            ARG_TARGET => account,
            ARG_ID => Option::<u64>::None,
        },
    )
    .build();
    builder.exec(transfer).expect_success().commit();
}
