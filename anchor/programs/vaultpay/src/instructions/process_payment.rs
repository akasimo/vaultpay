// instructions/process_payment.rs
use anchor_lang::prelude::*;
use anchor_spl::{    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, transfer_checked, TransferChecked}};

use crate::states::{Config, Subscription, Vendor, SubscriptionStatus};
use mock_yield_source::program::MockYieldSource;
use mock_yield_source::cpi::accounts::Withdraw as YieldSourceWithdraw;

#[derive(Accounts)]
pub struct ProcessPayment<'info> {
    #[account(mut)]
    pub vendor_signer: Signer<'info>,
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [b"config", token_mint.key().as_ref(), config.authority.key().as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    /// CHECK: This is a PDA used as a signer
    #[account(
        mut,
        seeds = [b"vaultpay_authority", config.key().as_ref(), subscription.user.key().as_ref()],
        bump
    )]
    pub vaultpay_authority: UncheckedAccount<'info>,

    /// CHECK: This is checked in the CPI to mock_yield_source
    #[account(mut)]
    pub yield_account: UncheckedAccount<'info>,
    
    /// CHECK: This is checked in the CPI to mock_yield_source
    #[account(mut)]
    pub yield_reserve: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"subscription", vendor.key().as_ref(), subscription.user.key().as_ref()],
        bump = subscription.bump,
    )]
    pub subscription: Account<'info, Subscription>,

    #[account(
        seeds = [b"vendor", config.key().as_ref(), vendor_signer.key().as_ref()],
        bump = vendor.bump,
    )]
    pub vendor: Account<'info, Vendor>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = yield_account,
    )]
    pub yield_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = vendor_signer,
    )]
    pub vendor_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = yield_reserve
    )]
    pub reserve_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = vaultpay_authority
    )]
    pub vaultpay_authority_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = config,
    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

    pub yield_program: Program<'info, MockYieldSource>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> ProcessPayment<'info> {
    pub fn process_payment(&mut self, bumps: &ProcessPaymentBumps) -> Result<()> {
        if self.subscription.status != SubscriptionStatus::Active {
            return Err(error!(crate::errors::VaultPayError::SubscriptionNotActive));
        }

        // Ensure subscription vendor is same as given account vendor
        if self.subscription.vendor != self.vendor.key() {
            return Err(error!(crate::errors::VaultPayError::InvalidVendor));
        }

        // Calculate platform fee and vendor amount
        let platform_fee = self.subscription.amount_per_payment
            .checked_mul(self.config.platform_fee as u64)
            .ok_or_else(|| error!(crate::errors::VaultPayError::MathOverflow))?
            .checked_div(10_000) // Assuming fee is in basis points
            .ok_or_else(|| error!(crate::errors::VaultPayError::MathOverflow))?;

        let amount_to_vendor = self.subscription
            .amount_per_payment
            .checked_sub(platform_fee)
            .ok_or_else(|| error!(crate::errors::VaultPayError::MathUnderflow))?;

        msg!("Platform Fee: {}", platform_fee);

        let binding_config = self.config.key();
        let binding_user_key = self.subscription.user.key();
        let seeds = &[
            b"vaultpay_authority",
            binding_config.as_ref(),
            binding_user_key.as_ref(),
            &[bumps.vaultpay_authority],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_program = self.yield_program.to_account_info();
        let cpi_accounts = YieldSourceWithdraw {
            user: self.vaultpay_authority.to_account_info(),
            token_mint: self.token_mint.to_account_info(),
            user_token_account: self.vaultpay_authority_ata.to_account_info(),
            yield_reserve: self.yield_reserve.to_account_info(),
            yield_account: self.yield_account.to_account_info(),
            yield_token_account: self.yield_token_account.to_account_info(),
            reserve_token_account: self.reserve_token_account.to_account_info(),
            token_program: self.token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        mock_yield_source::cpi::withdraw(cpi_ctx, self.subscription.amount_per_payment)?;
        
        // Get the token balance of the VaultPay authority's ATA
        let ata_balance = self.vaultpay_authority_ata.amount;
        msg!("VaultPay Authority ATA balance: {}", ata_balance);
        // Transfer to Vendor
        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.vaultpay_authority_ata.to_account_info(),
                    to: self.vendor_token_account.to_account_info(),
                    mint: self.token_mint.to_account_info(),
                    authority: self.vaultpay_authority.to_account_info(),
                },
                signer_seeds,
            ),
            amount_to_vendor,
            self.token_mint.decimals,
        )?;

        // Transfer platform fee to Treasury
        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.vaultpay_authority_ata.to_account_info(),
                    to: self.treasury_token_account.to_account_info(),
                    mint: self.token_mint.to_account_info(),
                    authority: self.vaultpay_authority.to_account_info(),
                },
                signer_seeds,
            ),
            platform_fee,
            self.token_mint.decimals,
        )?;

        // Update subscription
        self.subscription.payments_made += 1;

        if self.subscription.payments_made >= self.subscription.number_of_payments {
            self.subscription.status = SubscriptionStatus::Completed;
            msg!("Subscription status updated to Completed");
        }

        Ok(())
    }
}
