use casper_types::{account::AccountHash, EntityAddr, Key};

pub const CEP18_CONTRACT_WASM: &str = "cep18.wasm";
pub const CEP18_TEST_CONTRACT_WASM: &str = "cep18_test_contract.wasm";
pub const NAME_KEY: &str = "name";
pub const SYMBOL_KEY: &str = "symbol";
pub const CEP18_TOKEN_CONTRACT_KEY: &str = "cep18_contract_hash_CasperTest";
pub const CEP18_TOKEN_CONTRACT_VERSION_KEY: &str = "cep18_contract_version_CasperTest";
pub const DECIMALS_KEY: &str = "decimals";
pub const TOTAL_SUPPLY_KEY: &str = "total_supply";
pub const BALANCES_KEY: &str = "balances";
pub const ALLOWANCES_KEY: &str = "allowances";
pub const OWNER: &str = "owner";
pub const AMOUNT: &str = "amount";

pub const ARG_NAME: &str = "name";
pub const ARG_SYMBOL: &str = "symbol";
pub const ARG_DECIMALS: &str = "decimals";
pub const ARG_TOTAL_SUPPLY: &str = "total_supply";

pub const _ERROR_INVALID_CONTEXT: u16 = 60000;
pub const ERROR_INSUFFICIENT_BALANCE: u16 = 60001;
pub const ERROR_INSUFFICIENT_ALLOWANCE: u16 = 60002;
pub const ERROR_OVERFLOW: u16 = 60003;

pub const TOKEN_NAME: &str = "CasperTest";
pub const TOKEN_SYMBOL: &str = "CSPRT";
pub const TOKEN_DECIMALS: u8 = 100;
pub const TOKEN_TOTAL_SUPPLY: u64 = 1_000_000_000;

pub const METHOD_TRANSFER: &str = "transfer";
pub const ARG_AMOUNT: &str = "amount";
pub const ARG_RECIPIENT: &str = "recipient";

pub const METHOD_APPROVE: &str = "approve";
pub const ARG_OWNER: &str = "owner";
pub const ARG_SPENDER: &str = "spender";

pub const METHOD_TRANSFER_FROM: &str = "transfer_from";

pub const CHECK_TOTAL_SUPPLY_ENTRYPOINT: &str = "check_total_supply";
pub const CHECK_BALANCE_OF_ENTRYPOINT: &str = "check_balance_of";
pub const CHECK_ALLOWANCE_OF_ENTRYPOINT: &str = "check_allowance_of";
pub const ARG_TOKEN_CONTRACT: &str = "token_contract";
pub const ARG_ADDRESS: &str = "address";
pub const RESULT_KEY: &str = "result";
pub const CEP18_TEST_CONTRACT_KEY: &str = "cep18_test_contract";

pub const TRANSFER_AMOUNT_1: u64 = 200_001;
pub const TRANSFER_AMOUNT_2: u64 = 19_999;
pub const ALLOWANCE_AMOUNT_1: u64 = 456_789;
pub const ALLOWANCE_AMOUNT_2: u64 = 87_654;

pub const METHOD_TRANSFER_AS_STORED_CONTRACT: &str = "transfer_as_stored_contract";
pub const METHOD_APPROVE_AS_STORED_CONTRACT: &str = "approve_as_stored_contract";
pub const METHOD_FROM_AS_STORED_CONTRACT: &str = "transfer_from_as_stored_contract";

pub const TOKEN_OWNER_ADDRESS_1: Key = Key::AddressableEntity(EntityAddr::Account([42; 32]));
pub const TOKEN_OWNER_AMOUNT_1: u64 = 1_000_000;
pub const TOKEN_OWNER_ADDRESS_2: Key = Key::AddressableEntity(EntityAddr::SmartContract([42; 32]));
pub const TOKEN_OWNER_AMOUNT_2: u64 = 2_000_000;
pub const TOKEN_OWNER_ADDRESS_1_OLD: Key = Key::Account(AccountHash::new([42; 32]));
pub const _TOKEN_OWNER_ADDRESS_2_OLD: Key = Key::Hash([42; 32]);

pub const METHOD_MINT: &str = "mint";
pub const METHOD_BURN: &str = "burn";
pub const DECREASE_ALLOWANCE: &str = "decrease_allowance";
pub const INCREASE_ALLOWANCE: &str = "increase_allowance";
pub const ENABLE_MINT_BURN: &str = "enable_mint_burn";
pub const ADMIN_LIST: &str = "admin_list";
pub const MINTER_LIST: &str = "minter_list";
pub const NONE_LIST: &str = "none_list";
pub const CHANGE_SECURITY: &str = "change_security";

pub const USER_KEY_MAP: &str = "user_key_map";
pub const EVENTS: &str = "events";
pub const REVERT: &str = "revert";
pub const EVENTS_MODE: &str = "events_mode";
pub const MIGRATE_USER_BALANCE_KEYS_ENTRY_POINT_NAME: &str = "migrate_user_balance_keys";
pub const _MIGRATE_USER_ALLOWANCE_KEYS_ENTRY_POINT_NAME: &str = "migrate_user_allowance_keys";
pub const MIGRATE_USER_SEC_KEYS_ENTRY_POINT_NAME: &str = "migrate_sec_keys";
