use anchor_lang::prelude::*;

#[error_code]
pub enum VaultPayError {
    #[msg("The subscription is not active.")]
    SubscriptionNotActive,

    #[msg("Invalid Yield Account")]
    InvalidYieldAccount,
}
