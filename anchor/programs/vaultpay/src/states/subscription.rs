// states/subscription.rs
use anchor_lang::prelude::*;

#[account]
pub struct Subscription {
    pub user: Pubkey,
    pub vendor: Pubkey,
    pub seed: u64,
    pub start_time: i64,
    pub amount_per_payment: u64,
    pub number_of_payments: u8,
    pub payments_made: u8,
    pub status: SubscriptionStatus, // 1 byte
    pub locked: bool,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum SubscriptionStatus {
    Active,
    Cancelled,
    Completed,
}

impl Subscription {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 1 + 1 + 1 + 1 + 1;
}
