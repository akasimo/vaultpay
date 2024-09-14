use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use mock_yield_source::{program::MockYieldSource};
use mock_yield_source::cpi::accounts::OpenVault;
use mock_yield_source::states::{YieldReserve, YieldAccount};

use crate::errors::VaultPayError;

#[derive(Accounts)]
pub struct InitUser<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [b"yield_reserve", token_mint.key().as_ref()],
        bump = yield_reserve.bump
    )]
    pub yield_reserve: Account<'info, YieldReserve>,

    #[account(
        seeds = [b"vaultpay_authority", user.key().as_ref()],
        bump
    )]
    /// CHECK: This is a PDA used as a signer
    pub vaultpay_authority: UncheckedAccount<'info>,

    /// CHECK: cant check, because it ll be constrained with lending platforms programid
    pub yield_account: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = yield_account
    )]
    pub yield_token_account: InterfaceAccount<'info, TokenAccount>,

    pub yield_program: Program<'info, MockYieldSource>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitUser<'info> {
    pub fn init_user(&mut self, bumps: &InitUserBumps) -> Result<()> {
        msg!("Yield Account pubkey: {}", self.yield_account.key());

        let (yield_account_pda, yield_account_bump) = Pubkey::find_program_address(
            &[b"yield_account", self.yield_reserve.key().as_ref(), self.vaultpay_authority.key().as_ref()],
            &mock_yield_source::ID, // Use the program ID of mock_yield_source
        );
        msg!("Yield Account PDA pubkey: {}", yield_account_pda);

        require!(self.yield_account.key() == yield_account_pda, VaultPayError::InvalidYieldAccount);

        
        let cpi_program = self.yield_program.to_account_info();
        let cpi_accounts = OpenVault {
            user: self.user.to_account_info(),
            authority: self.vaultpay_authority.to_account_info(),
            token_mint: self.token_mint.to_account_info(),
            yield_reserve: self.yield_reserve.to_account_info(),
            yield_account: self.yield_account.to_account_info(),
            yield_token_account: self.yield_token_account.to_account_info(),
            token_program: self.token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
        };

        let seeds = &[
            b"vaultpay_authority",
            self.user.key.as_ref(),
            &[bumps.vaultpay_authority],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        mock_yield_source::cpi::open_vault(cpi_ctx, self.vaultpay_authority.key())?;
        Ok(())
    }
}
