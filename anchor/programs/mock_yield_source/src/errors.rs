use anchor_lang::error_code;

#[error_code]
pub enum MockYieldSourceError {
    #[msg("Invalid amount")]
    InvalidAmount,

    #[msg("Insufficient funds")]
    InsufficientFunds,

    #[msg("Invalid reserve")]
    InvalidReserve,
}