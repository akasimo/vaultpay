use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;
pub mod states;
pub mod helper;
pub mod errors;

declare_id!("EBpoUVQRJVrdYWzBgcDennNjfTsxaG8nspMAVCDoc1dx");

#[program]
pub mod mock_yield_source {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, apy: f64, initial_deposit: u64) -> Result<()> {
        ctx.accounts.init_yield_reserve(apy, initial_deposit, &ctx.bumps)?;
        Ok(())
    }

    pub fn open_vault(ctx: Context<OpenVault>, authority: Pubkey) -> Result<()> {
        ctx.accounts.open_vault(authority, &ctx.bumps)?;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)?;
        Ok(())
    }

    // pub fn claim(ctx: Context<Claim>) -> Result<()> {
    //     let yield_account = &mut ctx.accounts.yield_account;
    //     update_yield(
    //         yield_account,
    //         &ctx.accounts.yield_reserve,
    //         &ctx.accounts.reserve_token_account,
    //         &ctx.accounts.yield_token_account,
    //         &ctx.accounts.token_program,
    //     )?;

    //     let yield_amount = yield_account.unclaimed_yield;
    //     yield_account.unclaimed_yield = 0;

    //     let seeds = &[
    //         yield_account.to_account_info().key.as_ref(),
    //         &[yield_account.bump],
    //     ];
    //     let signer = &[&seeds[..]];

    //     token::transfer(
    //         CpiContext::new_with_signer(
    //             ctx.accounts.token_program.to_account_info(),
    //             Transfer {
    //                 from: ctx.accounts.yield_token_account.to_account_info(),
    //                 to: ctx.accounts.user_token_account.to_account_info(),
    //                 authority: yield_account.to_account_info(),
    //             },
    //             signer,
    //         ),
    //         yield_amount,
    //     )?;

    //     Ok(())
    // }
}


