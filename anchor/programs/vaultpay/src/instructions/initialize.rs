use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::states::Config;
use mock_yield_source::program::MockYieldSource;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    pub supported_token: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
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

    // #[account(
    //     init,
    //     payer = owner,
    //     associated_token::mint = supported_token,
    //     associated_token::authority = config,
    // )]
    // pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

    pub yield_program: Program<'info, MockYieldSource>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn init(
        &mut self,
        seed: u64,
        platform_fee: u16,
        min_subscription_duration: u64,
        max_subscription_duration: u64,
        bumps: &InitializeBumps,
    ) -> Result<()> {
        self.config.set_inner(Config {
            authority: self.owner.key(),
            seed,
            platform_fee,
            min_subscription_duration,
            max_subscription_duration,
            supported_token: self.supported_token.key(),
            yield_source: self.yield_program.key(),
            treasury_wallet: self.treasury.key(),
            locked: false,
            bump: bumps.config,
        });
        Ok(())
    }
}
