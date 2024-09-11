use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    pub supported_token: Account<'info, Token>,


    #[account(
        init, 
        payer = owner, 
        space = 8 + Config::INIT_SPACE,
        seeds = [b"treasury", owner.key().as_ref()],
        bump,
    )]
    pub treasury: Account<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn init(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}
