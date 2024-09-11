use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("EBpoUVQRJVrdYWzBgcDennNjfTsxaG8nspMAVCDoc1dx");

#[program]
pub mod mock_yield_source {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, apy: f64) -> Result<()> {
        let yield_account = &mut ctx.accounts.yield_account;
        yield_account.owner = ctx.accounts.owner.key();
        yield_account.token_mint = ctx.accounts.token_mint.key();
        yield_account.apy = apy;
        yield_account.last_update = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let yield_account = &mut ctx.accounts.yield_account;
        update_yield(yield_account)?;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.yield_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        yield_account.deposited_amount += amount;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let yield_account = &mut ctx.accounts.yield_account;
        update_yield(yield_account)?;

        let seeds = &[
            yield_account.to_account_info().key.as_ref(),
            &[yield_account.bump],
        ];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.yield_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: yield_account.to_account_info(),
                },
                signer,
            ),
            amount,
        )?;

        yield_account.deposited_amount = yield_account.deposited_amount.checked_sub(amount).unwrap();
        Ok(())
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let yield_account = &mut ctx.accounts.yield_account;
        update_yield(yield_account)?;

        let yield_amount = yield_account.unclaimed_yield;
        yield_account.unclaimed_yield = 0;

        let seeds = &[
            yield_account.to_account_info().key.as_ref(),
            &[yield_account.bump],
        ];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.yield_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: yield_account.to_account_info(),
                },
                signer,
            ),
            yield_amount,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_mint: Account<'info, token::Mint>,
    #[account(
        init,
        payer = owner,
        space = 8 + YieldAccount::LEN,
        seeds = [b"yield_account", token_mint.key().as_ref()],
        bump
    )]
    pub yield_account: Account<'info, YieldAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub yield_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub yield_account: Account<'info, YieldAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub yield_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub yield_account: Account<'info, YieldAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub yield_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub yield_account: Account<'info, YieldAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct YieldAccount {
    pub owner: Pubkey,
    pub token_mint: Pubkey,
    pub deposited_amount: u64,
    pub unclaimed_yield: u64,
    pub apy: f64,
    pub last_update: i64,
    pub bump: u8,
}

impl YieldAccount {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 8 + 1;
}

fn update_yield(yield_account: &mut Account<YieldAccount>) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let time_elapsed = (current_time - yield_account.last_update) as f64 / (365.0 * 24.0 * 60.0 * 60.0);
    let yield_rate = (1.0 + yield_account.apy).powf(time_elapsed) - 1.0;
    let new_yield = (yield_account.deposited_amount as f64 * yield_rate) as u64;
    yield_account.unclaimed_yield += new_yield;
    yield_account.last_update = current_time;
    Ok(())
}