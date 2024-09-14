// instructions/cancel_subscription.rs
use anchor_lang::prelude::*;
use crate::states::Subscription;

#[derive(Accounts)]
pub struct CancelSubscription<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"subscription", subscription.vendor.key().as_ref(), user.key().as_ref()],
        bump = subscription.bump,
        has_one = user,
    )]
    pub subscription: Account<'info, Subscription>,

    pub system_program: Program<'info, System>,
}

impl<'info> CancelSubscription<'info> {
    pub fn cancel_subscription(&mut self) -> Result<()> {
        require!(
            self.subscription.status == 0,
            crate::errors::ErrorCode::SubscriptionNotActive
        );
        self.subscription.status = 1; // Canceled
        Ok(())
    }
}
