use anchor_lang::prelude::*;

#[account]
pub struct YieldAccount {
    pub owner: Pubkey,
    pub yield_reserve: Pubkey,
    pub token_mint: Pubkey,
    pub deposited_amount: u64,
    pub unclaimed_yield: u64,
    pub last_update: i64,
    pub bump: u8,
}

impl YieldAccount {
    pub const LEN: usize = 32 + 32 + 32 + 8 + 8 + 8 + 1;
}