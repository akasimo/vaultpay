// states/subscription.rs
use anchor_lang::prelude::*;

#[account]
pub struct Subscription {
    pub authority: Pubkey,
    pub vendor: Pubkey,
    pub user: Pubkey,
    pub seed: u64,
    pub amount_per_payment: u64,
    pub number_of_payments: u8,
    pub payments_made: u8,
    pub start_time: i64,
    pub status: u8, // 0 = Active, 1 = Canceled, 2 = Completed
    pub locked: bool,
    pub bump: u8,
}

impl Subscription {
    pub const LEN: usize = 32 + 32 + 32 + 8 + 8 + 1 + 1 + 8 + 1 + 1 + 1;
}
