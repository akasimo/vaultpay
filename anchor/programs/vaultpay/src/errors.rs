use anchor_lang::prelude::*;

#[error_code]
pub enum VaultPayError {
    #[msg("The subscription is not active.")]
    SubscriptionNotActive,

    #[msg("Invalid Yield Account")]
    InvalidYieldAccount,

    #[msg("Invalid Yield Reserve")]
    InvalidYieldReserve,

    #[msg("Invalid Vendor")]
    InvalidVendor,

    #[msg("Math Underflow")]
    MathUnderflow,

    #[msg("Math Overflow")]
    MathOverflow,

    #[msg("Invalid Vault Pay Authority")]
    InvalidVaultPayAuthority,
}
