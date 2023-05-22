#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder,
        ARG_AMOUNT, DEFAULT_ACCOUNT_ADDR, DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
    };
    use casper_erc20::constants::{
        DECIMALS_RUNTIME_ARG_NAME, ERC20_TOKEN_CONTRACT_KEY_NAME, NAME_RUNTIME_ARG_NAME,
        SYMBOL_RUNTIME_ARG_NAME, TOTAL_SUPPLY_RUNTIME_ARG_NAME,
    };
    use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
    use casper_types::{
        account::AccountHash, runtime_args, Key, PublicKey, RuntimeArgs, SecretKey, URef, U256,
    };
    use erc20_token_sale::mods::constants::{
        TOKEN_SALE_CONTRACT_HASH, TOKEN_SALE_CONTRACT_PKG_HASH, TOKEN_SALE_CONTRACT_PKG_UREF,
        TOKEN_SALE_CONTRACT_VERSION_KEY,
    };

    const CONTRACT_WASM: &str = "erc20_token_sale.wasm";
    const TOKEN_WASM: &str = "erc20_token.wasm";

    #[test]
    fn should_install_contract() {
        let mut builder = InMemoryWasmTestBuilder::default();
        install_contract(&mut builder, TOKEN_WASM, get_token_args());
        let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        dbg!(account.named_keys());

        builder
            .query(
                None,
                Key::Account(*DEFAULT_ACCOUNT_ADDR),
                &[ERC20_TOKEN_CONTRACT_KEY_NAME.to_string()],
            )
            .expect("should be stored value.")
            .as_contract()
            .expect("should be contract hash.");

        install_contract(&mut builder, CONTRACT_WASM, runtime_args! {});
        let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
        dbg!(account.named_keys());

        builder
            .query(
                None,
                Key::Account(*DEFAULT_ACCOUNT_ADDR),
                &[TOKEN_SALE_CONTRACT_HASH.to_string()],
            )
            .expect("should be stored value.")
            .as_contract()
            .expect("should be contract hash.");

        builder
            .query(
                None,
                Key::Account(*DEFAULT_ACCOUNT_ADDR),
                &[TOKEN_SALE_CONTRACT_PKG_HASH.to_string()],
            )
            .expect("should be stored value.")
            .as_contract_package()
            .expect("should be contract hash.");

        builder
            .query(
                None,
                Key::Account(*DEFAULT_ACCOUNT_ADDR),
                &[TOKEN_SALE_CONTRACT_PKG_UREF.to_string()],
            )
            .expect("should be stored value.")
            .clone()
            .into_t::<URef>()
            .expect("should be URef.");

        let result_of_query = builder
            .query(
                None,
                Key::Account(*DEFAULT_ACCOUNT_ADDR),
                &[TOKEN_SALE_CONTRACT_VERSION_KEY.to_string()],
            )
            .expect("should be stored value.")
            .as_cl_value()
            .expect("should be cl value.")
            .clone()
            .into_t::<u32>()
            .expect("should be u32.");
        assert_eq!(result_of_query, 1_u32);
    }
    #[test]
    fn should_error_on_missing_runtime_arg() {
        const MY_ACCOUNT: [u8; 32] = [7u8; 32];
        let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);
        let account_addr = AccountHash::from(&public_key);

        let session_code = PathBuf::from(CONTRACT_WASM);
        let session_args = RuntimeArgs::new();

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_authorization_keys(&[account_addr])
            .with_address(*DEFAULT_ACCOUNT_ADDR)
            .with_session_code(session_code, session_args)
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();

        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
        builder.exec(execute_request).commit().expect_failure();
    }

    fn install_contract(
        builder: &mut WasmTestBuilder<InMemoryGlobalState>,
        wasm_file: &str,
        session_args: RuntimeArgs,
    ) {
        let session_code = PathBuf::from(wasm_file);
        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {
                ARG_AMOUNT => *DEFAULT_PAYMENT
            })
            .with_session_code(session_code, session_args)
            .with_authorization_keys(&[*DEFAULT_ACCOUNT_ADDR])
            .with_address(*DEFAULT_ACCOUNT_ADDR)
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();

        // Create a GenesisAccount.
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        // deploy the contract.
        builder.exec(execute_request).commit().expect_success();
    }

    fn get_token_args() -> RuntimeArgs {
        runtime_args! {
            NAME_RUNTIME_ARG_NAME => "TEST_TOKEN",
            SYMBOL_RUNTIME_ARG_NAME => "TEST",
            DECIMALS_RUNTIME_ARG_NAME=> 10_u8,
            TOTAL_SUPPLY_RUNTIME_ARG_NAME => U256::from(100_u8)
        }
    }
}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}
