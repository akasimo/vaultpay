use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, transfer_checked, TransferChecked},
};

use crate::states::Config;

#[derive(Accounts)]
pub struct ClaimTreasury<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    pub supported_token: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = supported_token,
        associated_token::authority = config,
    )]
    pub treasury: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = [b"config", supported_token.key().as_ref(), owner.key().as_ref()],
        bump,
        constraint = config.authority == owner.key(),
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        associated_token::mint = supported_token,
        associated_token::authority = config,
    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = supported_token,
        associated_token::authority = owner,
    )]
    pub owner_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> ClaimTreasury<'info> {
    pub fn claim_treasury(&mut self) -> Result<()> {
        // Transfer funds from Treasury to Authority
        // Check the balance of the treasury token account
        let treasury_balance = self.treasury_token_account.amount;
        
        // Ensure there are funds to transfer
        if treasury_balance == 0 {
            return Err(error!(crate::errors::VaultPayError::InsufficientFunds));
        }

        // Prepare the seeds for signing

        let binding_owner = self.owner.key();
        let binding_supported_token = self.supported_token.key();

        let config_seeds = &[
            b"config",
            binding_supported_token.as_ref(),
            binding_owner.as_ref(),
            &[self.config.bump],
        ];
        let signer_seeds = &[&config_seeds[..]];

        // Transfer the full balance from Treasury to Owner
        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.treasury_token_account.to_account_info(),
                    to: self.owner_token_account.to_account_info(),
                    mint: self.supported_token.to_account_info(),
                    authority: self.config.to_account_info(),
                },
                signer_seeds,
            ),
            treasury_balance,
            self.supported_token.decimals,
        )?;
        Ok(())
    }
}
