use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,
    pub seed: u64,
    pub platform_fee: u16,
    pub min_subscription_duration: u64,
    pub max_subscription_duration: u64,
    pub supported_token: Pubkey,
    pub yield_source: Pubkey,
    pub treasury_wallet: Pubkey,
    pub locked: bool,
    pub bump: u8,
}
