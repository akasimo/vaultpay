use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use mock_yield_source::program::MockYieldSource;
use mock_yield_source::cpi::accounts::Withdraw as YieldSourceWithdraw;
use mock_yield_source::states::{YieldReserve, YieldAccount};

#[derive(Accounts)]
pub struct Withdraw<'info> {
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

    #[account(
        mut,
        seeds = [b"yield_account", yield_reserve.key().as_ref(), vaultpay_authority.key().as_ref()],
        bump = yield_account.bump
    )]
    pub yield_account: Account<'info, YieldAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = user
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = yield_account
    )]
    pub yield_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = yield_reserve
    )]
    pub reserve_token_account: InterfaceAccount<'info, TokenAccount>,

    pub yield_program: Program<'info, MockYieldSource>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64, bump: u8) -> Result<()> {
        let cpi_program = self.yield_program.to_account_info();
        let cpi_accounts = YieldSourceWithdraw {
            user: self.vaultpay_authority.to_account_info(),
            token_mint: self.token_mint.to_account_info(),
            user_token_account: self.user_token_account.to_account_info(),
            yield_reserve: self.yield_reserve.to_account_info(),
            yield_account: self.yield_account.to_account_info(),
            yield_token_account: self.yield_token_account.to_account_info(),
            reserve_token_account: self.reserve_token_account.to_account_info(),
            token_program: self.token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
        };

        let seeds = &[
            b"vaultpay_authority",
            self.user.key.as_ref(),
            &[bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        mock_yield_source::cpi::withdraw(cpi_ctx, amount)?;

        Ok(())
    }
}
