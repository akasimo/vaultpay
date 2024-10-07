use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::states::{Config, Vendor};

#[derive(Accounts)]
pub struct InitVendor<'info> {
    #[account(mut)]
    pub vendor_signer: Signer<'info>,
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [b"config", token_mint.key().as_ref(), config.authority.key().as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = vendor_signer,
        space = 8 + Vendor::LEN,
        seeds = [b"vendor", config.key().as_ref(), vendor_signer.key().as_ref()],
        bump,
    )]
    pub vendor: Account<'info, Vendor>,

    #[account(
        init,
        payer = vendor_signer,
        associated_token::mint = token_mint,
        associated_token::authority = vendor,
    )]
    pub vendor_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitVendor<'info> {
    pub fn init_vendor(&mut self, seed: u64, bumps: &InitVendorBumps) -> Result<()> {
        self.vendor.set_inner( Vendor {
            authority: self.vendor_signer.key(),
            vendor_wallet: self.vendor_token_account.key(),
            seed,
            is_whitelisted: true,
            bump: bumps.vendor,
        });
        msg!("Vendor initialized");
        msg!("Vendor: {}", self.vendor.key().to_string());
        msg!("Vendor token account: {}", self.vendor_token_account.key().to_string());
        Ok(())
    }
}
