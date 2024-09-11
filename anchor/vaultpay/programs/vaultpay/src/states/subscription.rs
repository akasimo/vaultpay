use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Subscription {
    pub authority: Pubkey,
    pub vendor: Pubkey,
    pub user: Pubkey,
    pub seed: u64,
    pub amount_per_payment: u64,
    pub number_of_payments: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub status: u8,
    pub locked: bool
}
