//! Error codes for the factory contract.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FactoryError {
    /// accept_admin called but no nomination is pending.
    NoPendingAdmin = 1,
    /// Fee amount is negative.
    InvalidFeeAmount = 2,
    /// Fee amount exceeds MAX_FEE_AMOUNT.
    FeeOutOfBounds = 3,
    /// Fee amount is below the configured MinFee.
    FeeBelowMin = 4,
    /// set_fee_enabled(true) called before token+dest+standard were set.
    FeeNotConfigured = 5,
    /// Custom fee expiry timestamp is not in the future.
    ExpiryInPast = 6,
}
