use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, transfer_checked, TransferChecked},
};

use crate::{errors::MockYieldSourceError, states::{YieldAccount, YieldReserve}};
use crate::helper::update_yield;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_mint: InterfaceAccount<'info, Mint>,

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
        seeds = [b"yield_account", yield_reserve.key().as_ref(), user.key().as_ref()],
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

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        update_yield(
            &mut self.yield_account,
            &self.yield_reserve,
            &self.reserve_token_account,
            &self.yield_token_account,
            &self.token_program,
            &self.token_mint
        )?;

        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.user_token_account.to_account_info(),
                    to: self.yield_token_account.to_account_info(),
                    mint: self.token_mint.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            amount,
            self.token_mint.decimals,
        )?;

        // self.yield_account.deposited_amount += amount;
        self.yield_account.deposited_amount = self
            .yield_account
            .deposited_amount
            .checked_sub(amount)
            .ok_or(MockYieldSourceError::InsufficientFunds)?;
        Ok(())
    }
}