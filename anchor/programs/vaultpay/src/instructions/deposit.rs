use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, transfer_checked, TransferChecked},
};

use mock_yield_source::program::MockYieldSource;
use mock_yield_source::cpi::accounts::Deposit as YieldSourceDeposit;
use crate::states::Config;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [b"config", token_mint.key().as_ref(), config.authority.key().as_ref()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// CHECK: This is a PDA used as a signer
    #[account(
        mut,
        seeds = [b"vaultpay_authority", config.key().as_ref(), user.key().as_ref()],
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
        associated_token::mint = token_mint,
        associated_token::authority = user
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: directing to yield platform
    #[account(mut)]
    pub yield_token_account: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = yield_reserve
    )]
    pub reserve_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = vaultpay_authority
    )]
    pub vaultpay_authority_ata: InterfaceAccount<'info, TokenAccount>,

    pub yield_program: Program<'info, MockYieldSource>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64, bumps: &DepositBumps) -> Result<()> {
        // Transfer tokens from user to vaultpay_authority
        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.user_token_account.to_account_info(),
                    to: self.vaultpay_authority_ata.to_account_info(),
                    mint: self.token_mint.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            amount,
            self.token_mint.decimals,
        )?;

        msg!("Vaultpay authority ATA: {}", self.vaultpay_authority_ata.key());

        msg!("Transferred tokens from user to vaultpay_authority");

        // Prepare CPI to mock_yield_source deposit
        let cpi_program = self.yield_program.to_account_info();
        let cpi_accounts = YieldSourceDeposit {
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

        let binding_config = self.config.key();
        let seeds = &[
            b"vaultpay_authority",
            binding_config.as_ref(),
            self.user.key.as_ref(),
            &[bumps.vaultpay_authority],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        mock_yield_source::cpi::deposit(cpi_ctx, amount)?;

        Ok(())
    }
}
