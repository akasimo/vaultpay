use anchor_lang::prelude::*;
use crate::states::{Subscription, SubscriptionStatus, Config, Vendor};
use anchor_spl::{
    token_interface::{Mint},
};

#[derive(Accounts)]
pub struct InitSubscription<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [b"config", token_mint.key().as_ref(), config.authority.key().as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [b"vendor", config.key().as_ref(), vendor.authority.key().as_ref()],
        bump = vendor.bump,
    )]
    pub vendor: Account<'info, Vendor>,

    #[account(
        init,
        payer = user,
        space = 8 + Subscription::LEN,
        seeds = [b"subscription", vendor.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub subscription: Account<'info, Subscription>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitSubscription<'info> {
    pub fn init_subscription(
        &mut self,
        seed: u64,
        amount_per_payment: u64,
        number_of_payments: u8,
        start_time: i64,
        bumps: &InitSubscriptionBumps,
    ) -> Result<()> {
        self.subscription.set_inner(Subscription {
            user: self.user.key(),
            vendor: self.vendor.key(),
            seed,
            start_time,
            amount_per_payment,
            number_of_payments,
            payments_made: 0,
            status: SubscriptionStatus::Active,
            locked: false,
            bump: bumps.subscription,
        });
        Ok(())
    }
}
