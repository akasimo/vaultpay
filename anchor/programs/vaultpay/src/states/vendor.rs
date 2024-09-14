// states/vendor.rs
use anchor_lang::prelude::*;

#[account]
pub struct Vendor {
    pub authority: Pubkey,
    pub vendor_wallet: Pubkey,
    pub seed: u64,
    pub is_whitelisted: bool,
    pub bump: u8,
}

impl Vendor {
    pub const LEN: usize = 32 + 32 + 8 + 1 + 1;
}
