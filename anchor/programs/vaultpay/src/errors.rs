use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("The subscription is not active.")]
    SubscriptionNotActive,
}
