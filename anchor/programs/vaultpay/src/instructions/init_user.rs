use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenInterface},
};
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

use mock_yield_source::{program::MockYieldSource};
use mock_yield_source::cpi::accounts::OpenVault;
use crate::states::Config;
use crate::errors::VaultPayError;

#[derive(Accounts)]
pub struct InitUser<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,
    
    #[account(
        seeds = [b"config", token_mint.key().as_ref(), config.authority.key().as_ref()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        // init_if_needed,
        // payer = user,
        mut,
        seeds = [b"vaultpay_authority", config.key().as_ref(), user.key().as_ref()],
        bump
    )]
    /// CHECK: This is a PDA used as a signer
    pub vaultpay_authority: UncheckedAccount<'info>,
    
    /// CHECK: directing to yield platform
    #[account(
        mut
    )]
    pub yield_account: UncheckedAccount<'info>,

    /// CHECK: directing to yield platform
    #[account(
        mut
    )]
    pub yield_reserve: UncheckedAccount<'info>,
    
    /// CHECK: directing to yield platform
    #[account(
        mut
    )]
    pub yield_token_account: UncheckedAccount<'info>,

    pub yield_program: Program<'info, MockYieldSource>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitUser<'info> {
    pub fn init_user(&mut self, bumps: &InitUserBumps) -> Result<()> {
        let min_required_balance = LAMPORTS_PER_SOL / 100;
        let target_balance = min_required_balance * 3; // Equivalent to 3% of 1 SOL

        let current_balance = self.vaultpay_authority.lamports();
        if current_balance < min_required_balance {
            let amount_to_transfer = target_balance - current_balance;
            
            // Transfer SOL from user to vaultpay_authority
            system_program::transfer(
                CpiContext::new(
                    self.system_program.to_account_info(),
                    system_program::Transfer {
                        from: self.user.to_account_info(),
                        to: self.vaultpay_authority.to_account_info(),
                    },
                ),
                amount_to_transfer,
            )?;
        }

        msg!("Yield Account pubkey: {}", self.yield_account.key());
        let (yield_account_pda, _yield_account_bump) = Pubkey::find_program_address(
            &[b"yield_account", self.yield_reserve.key().as_ref(), self.vaultpay_authority.key().as_ref()],
            &mock_yield_source::ID, // Use the program ID of mock_yield_source
        );

        msg!("Yield Account PDA pubkey: {}", yield_account_pda);
        require!(self.yield_account.key() == yield_account_pda, VaultPayError::InvalidYieldAccount);

        msg!("Yield Reserve pubkey: {}", self.yield_reserve.key());
        let (_yield_reserve_pda, _yield_reserve_bump) = Pubkey::find_program_address(
            &[b"yield_reserve", self.token_mint.key().as_ref()],
            &mock_yield_source::ID, // Use the program ID of mock_yield_source
        );
        msg!("Yeild Reserve PDA pubkey: {}", _yield_reserve_pda);
        require!(self.yield_reserve.key() == _yield_reserve_pda, VaultPayError::InvalidYieldReserve);
        
        let cpi_program = self.yield_program.to_account_info();
        let cpi_accounts = OpenVault {
            user: self.vaultpay_authority.to_account_info(),
            token_mint: self.token_mint.to_account_info(),
            yield_reserve: self.yield_reserve.to_account_info(),
            yield_account: self.yield_account.to_account_info(),
            yield_token_account: self.yield_token_account.to_account_info(),
            token_program: self.token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
        };

        let binding_config = self.config.key();

        let seeds = &[
            b"vaultpay_authority",
            binding_config.as_ref(),
            self.user.key.as_ref(),
            &[bumps.vaultpay_authority],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        mock_yield_source::cpi::open_vault(cpi_ctx)?;
        Ok(())
    }
}
