use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Token, TokenAccount, Transfer},
};

declare_id!("EBpoUVQRJVrdYWzBgcDennNjfTsxaG8nspMAVCDoc1dx");

#[program]
pub mod mock_yield_source {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, apy: f64, initial_deposit: u64) -> Result<()> {
        let yield_reserve = &mut ctx.accounts.yield_reserve;
        yield_reserve.authority = ctx.accounts.authority.key();
        yield_reserve.token_mint = ctx.accounts.token_mint.key();
        yield_reserve.reserve_account = ctx.accounts.reserve_token_account.key();
        yield_reserve.apy = apy;
        yield_reserve.bump = ctx.bumps.yield_reserve;

        // Transfer initial deposit to reserve token account
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.authority_token_account.to_account_info(),
                    to: ctx.accounts.reserve_token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            initial_deposit,
        )?;

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let yield_account = &mut ctx.accounts.yield_account;
        update_yield(
            yield_account,
            &ctx.accounts.yield_reserve,
            &ctx.accounts.reserve_token_account,
            &ctx.accounts.yield_token_account,
            &ctx.accounts.token_program,
        )?;

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
        update_yield(
            yield_account,
            &ctx.accounts.yield_reserve,
            &ctx.accounts.reserve_token_account,
            &ctx.accounts.yield_token_account,
            &ctx.accounts.token_program,
        )?;

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

        yield_account.deposited_amount =
            yield_account.deposited_amount.checked_sub(amount).unwrap();
        Ok(())
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let yield_account = &mut ctx.accounts.yield_account;
        update_yield(
            yield_account,
            &ctx.accounts.yield_reserve,
            &ctx.accounts.reserve_token_account,
            &ctx.accounts.yield_token_account,
            &ctx.accounts.token_program,
        )?;

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

    pub fn open_vault(ctx: Context<OpenVault>) -> Result<()> {
        let yield_account = &mut ctx.accounts.yield_account;
        yield_account.owner = ctx.accounts.user.key();
        yield_account.yield_reserve = ctx.accounts.yield_reserve.key();
        yield_account.token_mint = ctx.accounts.token_mint.key();
        yield_account.deposited_amount = 0;
        yield_account.unclaimed_yield = 0;
        yield_account.last_update = Clock::get()?.unix_timestamp;
        yield_account.bump = ctx.bumps.yield_account;
        Ok(())
    }
}

#[account]
pub struct YieldAccount {
    pub owner: Pubkey,
    pub yield_reserve: Pubkey,
    pub token_mint: Pubkey,
    pub deposited_amount: u64,
    pub unclaimed_yield: u64,
    pub last_update: i64,
    pub bump: u8,
}

impl YieldAccount {
    pub const LEN: usize = 32 + 32 + 32 + 8 + 8 + 8 + 1;
}

#[account]
pub struct YieldReserve {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub reserve_account: Pubkey,
    pub apy: f64,
    pub bump: u8,
}

impl YieldReserve {
    pub const LEN: usize = 32 + 32 + 32 + 8 + 1;
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub token_mint: Account<'info, token::Mint>,
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
    pub reserve_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub authority_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
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

    #[account(mut)]
    pub yield_reserve: Account<'info, YieldReserve>,

    pub token_mint: Account<'info, token::Mint>,
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = yield_reserve
    )]
    pub reserve_token_account: Account<'info, TokenAccount>,
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
    #[account(mut)]
    pub yield_reserve: Account<'info, YieldReserve>,

    pub token_mint: Account<'info, token::Mint>,
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = yield_reserve
    )]
    pub reserve_token_account: Account<'info, TokenAccount>,
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
    #[account(mut)]
    pub yield_reserve: Account<'info, YieldReserve>,

    pub token_mint: Account<'info, token::Mint>,
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = yield_reserve
    )]
    pub reserve_token_account: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct OpenVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub yield_reserve: Account<'info, YieldReserve>,
    pub token_mint: Account<'info, token::Mint>,
    #[account(
        init,
        payer = user,
        space = 8 + YieldAccount::LEN,
        seeds = [b"yield_account", yield_reserve.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub yield_account: Account<'info, YieldAccount>,
    #[account(
        init,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = yield_account
    )]
    pub yield_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

fn update_yield<'info>(
    yield_account: &mut Account<'info, YieldAccount>,
    yield_reserve: &Account<'info, YieldReserve>,
    reserve_token_account: &Account<'info, TokenAccount>,
    yield_token_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
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

    token::transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: reserve_token_account.to_account_info(),
                to: yield_token_account.to_account_info(),
                authority: yield_reserve.to_account_info(),
            },
            signer,
        ),
        new_yield,
    )?;

    yield_account.unclaimed_yield += new_yield;
    yield_account.last_update = current_time;
    Ok(())
}
