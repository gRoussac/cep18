pub mod constants;

use casper_types::ApiError;

#[non_exhaustive]
pub enum InvestingError {
    InvalidKeyVariant,
    MissingDepositPurseURef,
    MissingLedgerSeedURef,
    MissingBalancesSeedURef,
    MissingCountInvestorsKey,
    MissingCountInvestmentsKey,
    MissingERC20TokenURef,
    MissingOwnerHash,
    Test,
}

impl From<InvestingError> for ApiError {
    fn from(code: InvestingError) -> Self {
        ApiError::User(code as u16)
    }
}
