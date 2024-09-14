use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::states::{YieldReserve, YieldAccount};


#[derive(Accounts)]
pub struct OpenVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_mint: InterfaceAccount<'info, Mint>,
    
    /// CHECK: Can be a PDA
    #[account(
        mut,
        signer
    )]
    pub authority: UncheckedAccount<'info>,

    #[account(
        seeds = [b"yield_reserve", token_mint.key().as_ref()],
        bump
    )]
    pub yield_reserve: Account<'info, YieldReserve>,

    #[account(
        init,
        payer = user,
        space = 8 + YieldAccount::LEN,
        seeds = [b"yield_account", yield_reserve.key().as_ref(), authority.key().as_ref()],
        bump
    )]
    pub yield_account: Account<'info, YieldAccount>,

    #[account(
        init,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = yield_account
    )]
    pub yield_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> OpenVault<'info> {
    pub fn open_vault(&mut self, authority: Pubkey, bumps: &OpenVaultBumps) -> Result<()> {
        self.yield_account.set_inner(YieldAccount {
            owner: self.user.key(),
            authority,
            // yield_reserve: self.yield_reserve.key(),
            token_mint: self.token_mint.key(),
            deposited_amount: 0,
            unclaimed_yield: 0,
            last_update: Clock::get()?.unix_timestamp,
            bump: bumps.yield_account,
        });
        Ok(())
    }
}