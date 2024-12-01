use crate::utility::{
    constants::{ACCOUNT_USER_1, TOKEN_TOTAL_SUPPLY},
    installer_request_builders::{cep18_check_balance_of, setup, TestContext},
};
use casper_engine_test_support::{ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR};
use casper_types::{runtime_args, Key, RuntimeArgs, U256};
use cowl_cep18::constants::{ARG_AMOUNT, ARG_RECIPIENT, ARG_TOTAL_SUPPLY, ENTRY_POINT_ALLOCATE};

#[test]
fn should_allocate_full_owned_amount() {
    let (
        mut builder,
        TestContext {
            cep18_token,
            ref test_accounts,
            ..
        },
    ) = setup();

    let account_user_1 = *test_accounts.get(&ACCOUNT_USER_1).unwrap();

    let initial_supply = U256::from(TOKEN_TOTAL_SUPPLY);
    let allocate_amount_1 = initial_supply;

    let allocate_1_sender = *DEFAULT_ACCOUNT_ADDR;
    let cep18_allocate_1_args = runtime_args! {
        ARG_RECIPIENT => Key::Account(account_user_1),
        ARG_AMOUNT => allocate_amount_1,
    };

    let owner_balance_before = cep18_check_balance_of(
        &mut builder,
        &cep18_token,
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
    );
    assert_eq!(owner_balance_before, initial_supply);

    let account_1_balance_before =
        cep18_check_balance_of(&mut builder, &cep18_token, Key::Account(account_user_1));
    assert_eq!(account_1_balance_before, U256::zero());

    let token_allocate_request_1 = ExecuteRequestBuilder::contract_call_by_hash(
        allocate_1_sender,
        cep18_token,
        ENTRY_POINT_ALLOCATE,
        cep18_allocate_1_args,
    )
    .build();

    builder
        .exec(token_allocate_request_1)
        .expect_success()
        .commit();

    let account_1_balance_after =
        cep18_check_balance_of(&mut builder, &cep18_token, Key::Account(account_user_1));
    assert_eq!(account_1_balance_after, allocate_amount_1);

    let owner_balance_after = cep18_check_balance_of(
        &mut builder,
        &cep18_token,
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
    );
    assert_eq!(owner_balance_after, U256::zero());

    let total_supply: U256 = builder.get_value(cep18_token, ARG_TOTAL_SUPPLY);
    assert_eq!(total_supply, initial_supply);
}
