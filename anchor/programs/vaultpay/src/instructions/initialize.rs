use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::states::Config;

use mock_yield_source::program::MockYieldSource;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    pub supported_token: Box<InterfaceAccount<'info, Mint>>,
    
    #[account(
        init,
        payer = owner,
        associated_token::mint = supported_token,
        associated_token::authority = config,
    )]
    pub treasury: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        space = 8 + Config::INIT_SPACE,
        payer = owner, 
        seeds = [b"config", supported_token.key().as_ref(), owner.key().as_ref()],
        bump,
    )]
    pub config: Account<'info, Config>,

    pub yield_source: Program<'info, MockYieldSource>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn init(&mut self, ctx: Context<Initialize>, seed: u64, platform_fee: u16, min_subscription_duration: u64, max_subscription_duration: u64, bumps: &InitializeBumps) -> Result<()> {
        self.config.set_inner( Config {
            authority : ctx.accounts.owner.key(),
            seed,
            platform_fee,
            min_subscription_duration : min_subscription_duration,
            max_subscription_duration : max_subscription_duration,
            supported_token : ctx.accounts.supported_token.key(),
            yield_source : ctx.accounts.yield_source.key(),
            treasury_wallet : ctx.accounts.treasury.key(),
            locked : false,
            bump: bumps.config,
        });
        Ok(())
    }
}
