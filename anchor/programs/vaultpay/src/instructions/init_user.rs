use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenInterface},
};

use mock_yield_source::program::MockYieldSource;
use mock_yield_source::cpi::accounts::OpenVault;

#[derive(Accounts)]
pub struct InitUser<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [b"vaultpay_authority", user.key().as_ref()],
        bump,
    )]
    /// CHECK: This is a PDA used as a signer
    pub vaultpay_authority: UncheckedAccount<'info>,

    

    pub yield_program: Program<'info, MockYieldSource>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitUser<'info> {
    pub fn init_user(&mut self, ctx: Context<InitUser>) -> Result<()> {
        let cpi_program = ctx.accounts.yield_program.to_account_info();
        let cpi_accounts = OpenVault {
            user: self.vaultpay_authority.key,
            yield_reserve: todo!(),
            token_mint: self.token_mint.key(),
            yield_account: todo!(),
            yield_token_account: todo!(),
            token_program: todo!(),
            system_program: todo!(),
            associated_token_program: todo!(),
        };
    }
}