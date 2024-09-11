use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;
pub mod states;
pub mod errors;

declare_id!("BpNWEQeYpjpA9w9EGXEfmHoLhb1nrYcm83wTptbHgdh5");

#[program]
pub mod vaultpay {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn init_user(ctx: Context<InitUserVault>) -> Result<()> {
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>) -> Result<()> {
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        Ok(())
    }

    pub fn init_vendor(ctx: Context<InitVendorVault>) -> Result<()> {
        Ok(())
    }

    pub fn init_subscription(ctx: Context<InitSubscription>) -> Result<()> {
        Ok(())
    }

    pub fn cancel_subscription(ctx: Context<CancelSubscription>) -> Result<()> {
        Ok(())
    }

    pub fn process_payment(ctx: Context<ProcessPayment>) -> Result<()> {
        Ok(())
    }

    pub fn claim_treasury(ctx: Context<ClaimTreasury>) -> Result<()> {
        Ok(())
    }
}