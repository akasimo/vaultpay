// instructions/claim_treasury.rs
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface, transfer_checked, TransferChecked};

use crate::states::Config;

#[derive(Accounts)]
pub struct ClaimTreasury<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub supported_token: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [b"config", supported_token.key().as_ref(), authority.key().as_ref()],
        bump = config.bump,
        has_one = authority,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        associated_token::mint = supported_token,
        associated_token::authority = config,
    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = supported_token,
        associated_token::authority = authority,
    )]
    pub authority_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> ClaimTreasury<'info> {
    pub fn claim_treasury(&mut self, amount: u64) -> Result<()> {
        // Transfer funds from Treasury to Authority
        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.treasury_token_account.to_account_info(),
                    to: self.authority_token_account.to_account_info(),
                    mint: self.supported_token.to_account_info(),
                    authority: self.config.to_account_info(),
                },
            ),
            amount,
            self.supported_token.decimals,
        )?;
        Ok(())
    }
}
