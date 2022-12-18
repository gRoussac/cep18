//! Constants used by the token-sale-* contracts.

// Creating constants for the various contract entry points.
pub const ENTRY_POINT_INIT: &str = "init";

pub const ENTRY_POINT_INVEST: &str = "invest";

pub const ENTRY_POINT_COUNTERS: &str = "get_counters";

// Creating constants for values within the contract.

pub const TOKEN_SALE_CONTRACT_HASH: &str = "token_sale_contract_hash";

pub const TOKEN_SALE_CONTRACT_PKG_HASH: &str = "token_sale_package_hash";

pub const TOKEN_SALE_CONTRACT_PKG_UREF: &str = "token_sale_package_uref";

pub const TOKEN_SALE_CONTRACT_VERSION_KEY: &str = "token_sale_version";

// Sale configuration

pub const DEPOSIT_PURSE: &str = "deposit_purse";

pub const TOKEN_PRICE_IN_CSPR: u64 = ONE;

// Account Key from the caller investing

pub const INVEST_ACCOUNT_KEY: &str = "invest_account_key";

// Amount from the caller investing

pub const INVESTING_AMOUNT: &str = "amount";

// Counters named keys

pub const COUNT_INVESTORS_KEY: &str = "count_investors";

pub const COUNT_INVESTMENTS_KEY: &str = "count_investments";

pub const LEDGER: &str = "ledger";

pub const ZERO: u64 = 0_u64;

pub const ONE: u64 = 1_u64;
