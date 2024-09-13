use anchor_lang::prelude::*;

#[account]
pub struct YieldReserve {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub reserve_account: Pubkey,
    pub apy: f64,
    pub bump: u8,
}

impl YieldReserve {
    pub const LEN: usize = 32 + 32 + 32 + 8 + 1;
}