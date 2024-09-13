// use anchor_lang::prelude::*;
// use anchor_spl::{
//     associated_token::AssociatedToken,
//     token_interface::{Mint, TokenAccount, TokenInterface, transfer_checked, TransferChecked},
// };

// use crate::states::{YieldReserve, YieldAccount};
// use crate::helper::update_yield;

// #[derive(Accounts)]
// pub struct Claim<'info> {
//     #[account(mut)]
//     pub user: Signer<'info>,

//     pub token_mint: InterfaceAccount<'info, Mint>,

//     #[account(
//         mut,
//         associated_token::mint = token_mint,
//         associated_token::authority = user
//     )]
//     pub user_token_account: Interface<'info, TokenAccount>,

//     #[account(
//         mut,
//         seeds = [b"yield_reserve", token_mint.key().as_ref()],
//         bump
//     )]
//     pub yield_reserve: Account<'info, YieldReserve>,

//     #[account(
//         mut,
//         seeds = [b"yield_account", yield_reserve.key().as_ref(), user.key().as_ref()],
//         bump = yield_account.bump
//     )]
//     pub yield_account: Account<'info, YieldAccount>,

//     #[account(
//         mut,
//         associated_token::mint = token_mint,
//         associated_token::authority = yield_account
//     )]
//     pub yield_token_account: Interface<'info, TokenAccount>,

//     pub token_program: Interface<'info, TokenInterface>,
//     pub associated_token_program: Program<'info, AssociatedToken>,
//     pub system_program: Program<'info, System>,
// }