// instructions/process_payment.rs
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface, transfer_checked, TransferChecked};

use crate::states::{Config, Subscription, Vendor, SubscriptionStatus};
use mock_yield_source::states::YieldReserve;

#[derive(Accounts)]
pub struct ProcessPayment<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub supported_token: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [b"config", supported_token.key().as_ref(), config.authority.key().as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [b"subscription", vendor.key().as_ref(), subscription.user.key().as_ref()],
        bump = subscription.bump,
    )]
    pub subscription: Account<'info, Subscription>,

    #[account(
        seeds = [b"vendor", config.key().as_ref(), vendor.authority.key().as_ref()],
        bump = vendor.bump,
    )]
    pub vendor: Account<'info, Vendor>,

    /// CHECK: Vendor authority public key
    pub vendor_authority: UncheckedAccount<'info>,

    /// CHECK: User public key
    pub user: UncheckedAccount<'info>,

    #[account(
        seeds = [b"vaultpay_authority", subscription.user.key().as_ref()],
        bump,
    )]
    /// CHECK: PDA used as a signer
    pub vaultpay_authority: UncheckedAccount<'info>,

    #[account(
        seeds = [b"yield_reserve", supported_token.key().as_ref()],
        bump = yield_reserve.bump,
    )]
    pub yield_reserve: Account<'info, YieldReserve>,

    /// CHECK: cant check, because it ll be constrained with lending platforms programid
    pub yield_account: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = supported_token,
        associated_token::authority = yield_account,
    )]
    pub yield_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = supported_token,
        associated_token::authority = vendor,
    )]
    pub vendor_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = supported_token,
        associated_token::authority = config,
    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> ProcessPayment<'info> {
    pub fn process_payment(&mut self, bumps: &ProcessPaymentBumps) -> Result<()> {
        // Ensure subscription is active
        if self.subscription.status != SubscriptionStatus::Active {
            return Err(error!(crate::errors::VaultPayError::SubscriptionNotActive));
        }

        // Calculate platform fee and vendor amount
        let platform_fee = self.subscription.amount_per_payment
            .checked_mul(self.config.platform_fee as u64)
            .unwrap()
            .checked_div(10_000) // Assuming fee is in basis points
            .unwrap();

        let amount_to_vendor = self.subscription.amount_per_payment
            .checked_sub(platform_fee)
            .unwrap();

        let vaultpay_authority_bump = bumps.vaultpay_authority  ;
        let seeds = &[
            b"vaultpay_authority",
            self.subscription.user.as_ref(),
            &[vaultpay_authority_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        // Transfer to Vendor
        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.yield_token_account.to_account_info(),
                    to: self.vendor_token_account.to_account_info(),
                    mint: self.supported_token.to_account_info(),
                    authority: self.vaultpay_authority.to_account_info(),
                },
                signer_seeds,
            ),
            amount_to_vendor,
            self.supported_token.decimals,
        )?;

        // Transfer platform fee to Treasury
        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.yield_token_account.to_account_info(),
                    to: self.treasury_token_account.to_account_info(),
                    mint: self.supported_token.to_account_info(),
                    authority: self.vaultpay_authority.to_account_info(),
                },
                signer_seeds,
            ),
            platform_fee,
            self.supported_token.decimals,
        )?;

        // Update subscription
        self.subscription.payments_made += 1;

        if self.subscription.payments_made >= self.subscription.number_of_payments {
            self.subscription.status = SubscriptionStatus::Completed;
        }

        Ok(())
    }
}
