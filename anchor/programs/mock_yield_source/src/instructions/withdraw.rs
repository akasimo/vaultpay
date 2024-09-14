use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, transfer_checked, TransferChecked},
};

use crate::states::{YieldReserve, YieldAccount};
use crate::helper::update_yield;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub token_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: Can be a PDA
    #[account(
        mut,
        signer
    )]
    pub authority: UncheckedAccount <'info>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = user
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"yield_reserve", token_mint.key().as_ref()],
        bump
    )]
    pub yield_reserve: Account<'info, YieldReserve>,

    #[account(
        mut,
        has_one = authority,
        seeds = [b"yield_account", yield_reserve.key().as_ref(), authority.key().as_ref()],
        bump = yield_account.bump
    )]
    pub yield_account: Account<'info, YieldAccount>,

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

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        update_yield(
            &mut self.yield_account,
            &self.yield_reserve,
            &self.reserve_token_account,
            &self.yield_token_account,
            &self.token_program,
            &self.token_mint
        )?;

        let yield_account_balance = self.yield_token_account.amount;
        msg!("Yield account balance: {}", yield_account_balance);

        let seeds = &[
            b"yield_account",
            self.yield_reserve.to_account_info().key.as_ref(),
            self.authority.to_account_info().key.as_ref(),
            &[self.yield_account.bump],
        ];
        let signer = &[&seeds[..]];

        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.yield_token_account.to_account_info(),
                    to: self.user_token_account.to_account_info(),
                    mint: self.token_mint.to_account_info(),
                    authority: self.yield_account.to_account_info(),
                },
                signer,
            ),
            amount,
            self.token_mint.decimals,
        )?;

        self.yield_account.deposited_amount = self.yield_account.deposited_amount.checked_sub(amount).unwrap();
        Ok(())
    }
}