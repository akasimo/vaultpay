use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, transfer_checked, TransferChecked},
};
use crate::states::{YieldReserve, YieldAccount};

pub fn update_yield<'info>(
    yield_account: &mut Account<'info, YieldAccount>,
    yield_reserve: &Account<'info, YieldReserve>,
    reserve_token_account: &InterfaceAccount<'info, TokenAccount>,
    yield_token_account: &InterfaceAccount<'info, TokenAccount>,
    token_program: &Interface<'info, TokenInterface>,
    mint: &InterfaceAccount<'info, Mint>,
) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let time_elapsed =
        (current_time - yield_account.last_update) as f64 / (365.0 * 24.0 * 60.0 * 60.0);
    let yield_rate = (1.0 + yield_reserve.apy).powf(time_elapsed) - 1.0;
    let new_yield = (yield_account.deposited_amount as f64 * yield_rate) as u64;

    // Ensure the yield reserve has enough funds
    if reserve_token_account.amount < new_yield {
        return Err(ProgramError::InsufficientFunds.into());
    }

    // Transfer new yield from reserve to yield token account
    let seeds = &[
        b"yield_reserve",
        yield_reserve.token_mint.as_ref(),
        &[yield_reserve.bump],
    ];
    let signer = &[&seeds[..]];

    transfer_checked(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            TransferChecked {
                from: reserve_token_account.to_account_info(),
                to: yield_token_account.to_account_info(),
                mint: mint.to_account_info(),
                authority: yield_reserve.to_account_info(),
            },
            signer,
        ),
        new_yield,
        mint.decimals
    )?;

    yield_account.unclaimed_yield += new_yield;
    yield_account.last_update = current_time;
    Ok(())
}