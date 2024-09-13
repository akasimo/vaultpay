use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, transfer_checked, TransferChecked},
};

use crate::states::{YieldReserve, YieldAccount};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = authority,
    )]
    pub authority_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = authority,
        space = 8 + YieldReserve::LEN,
        seeds = [b"yield_reserve", token_mint.key().as_ref()],
        bump
    )]
    pub yield_reserve: Account<'info, YieldReserve>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = token_mint,
        associated_token::authority = yield_reserve
    )]
    pub reserve_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn init_yield_reserve(&mut self, apy: f64, initial_deposit: u64, bumps: &InitializeBumps) -> Result<()> {
        self.yield_reserve.set_inner(YieldReserve {
            authority: self.authority.key(),
            token_mint: self.token_mint.key(),
            reserve_account: self.reserve_token_account.key(),
            apy: apy,
            bump: bumps.yield_reserve,
        });

        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.authority_token_account.to_account_info(),
                    mint: self.token_mint.to_account_info(),
                    to: self.reserve_token_account.to_account_info(),
                    authority: self.authority.to_account_info(),
                },
            ),
            initial_deposit,
            self.token_mint.decimals,
        )?;

        Ok(())
    }
}