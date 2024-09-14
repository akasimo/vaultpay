use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;
pub mod states;
pub mod errors;

declare_id!("BpNWEQeYpjpA9w9EGXEfmHoLhb1nrYcm83wTptbHgdh5");

#[program]
pub mod vaultpay {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        seed: u64,
        platform_fee: u16,
        min_subscription_duration: u64,
        max_subscription_duration: u64,
    ) -> Result<()> {
        ctx.accounts.init(
            seed,
            platform_fee,
            min_subscription_duration,
            max_subscription_duration,
            &ctx.bumps,
        )?;
        Ok(())
    }

    pub fn init_user(ctx: Context<InitUser>) -> Result<()> {
        ctx.accounts.init_user(&ctx.bumps)?;
        Ok(())
    }

    pub fn init_vendor(ctx: Context<InitVendor>, seed: u64, is_whitelisted: bool) -> Result<()> {
        ctx.accounts.init_vendor(seed, is_whitelisted, &ctx.bumps)?;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount, &ctx.bumps)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount, &ctx.bumps)?;
        Ok(())
    }

    pub fn init_subscription(
        ctx: Context<InitSubscription>,
        seed: u64,
        amount_per_payment: u64,
        number_of_payments: u8,
        start_time: i64,
    ) -> Result<()> {
        ctx.accounts.init_subscription(
            seed,
            amount_per_payment,
            number_of_payments,
            start_time,
            &ctx.bumps,
        )?;
        Ok(())
    }

    pub fn process_payment(ctx: Context<ProcessPayment>) -> Result<()> {
        ctx.accounts.process_payment(&ctx.bumps)?;
        Ok(())
    }

    pub fn cancel_subscription(ctx: Context<CancelSubscription>) -> Result<()> {
        ctx.accounts.cancel_subscription()?;
        Ok(())
    }

    pub fn claim_treasury(ctx: Context<ClaimTreasury>, amount: u64) -> Result<()> {
        ctx.accounts.claim_treasury(amount)?;
        Ok(())
    }
}