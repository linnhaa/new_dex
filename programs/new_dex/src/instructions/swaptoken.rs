use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, TokenAccount, Mint};
use anchor_lang::solana_program::system_instruction;
use anchor_lang::solana_program::program::invoke_signed;
use crate::initialize::Pool;

#[derive(Accounts)]
pub struct SwapToken<'info> {
    #[account(
        mut,
        seeds = [b"POOL"],
        bump,
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(mut)]
    pub trader: Signer<'info>,

    #[account(mut)]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = trader_token_account.owner == trader.key(),
        constraint = trader_token_account.mint == pool.token_mint.key()
    )]
    pub trader_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = token_vault.owner == pool.key(),
        constraint = token_vault.mint == pool.token_mint.key()
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut, 
        seeds = [b"sol_vault", pool.key().as_ref()], 
        bump)]
    pub sol_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn swap_token(
    ctx: Context<SwapToken>,
    token_amount: u64
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let trader = &ctx.accounts.trader;

    let sol_balance = ctx.accounts.sol_vault.lamports();
    let token_balance = ctx.accounts.token_vault.amount;
    let k = sol_balance as u128 * token_balance as u128;

    // Calculate token amount using AMM formula
    let new_token_balance = token_balance as u128 + token_amount as u128;
    let new_sol_balance = k / new_token_balance;
    let sol_amount = sol_balance - new_sol_balance as u64;

    // Ensure pool has enough tokens for the swap
    require!(sol_balance >= sol_amount, ErrorCode::InsufficientPoolSolBalance);
    require!(ctx.accounts.trader_token_account.amount >= token_amount, ErrorCode::InsufficientTokenBalance);

    // Transfer Tokens to token_vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.trader_token_account.to_account_info(),
        to: ctx.accounts.token_vault.to_account_info(),
        authority: trader.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, token_amount)?;

    // Transfer SOL from sol_vault back to LP
    let transfer_instruction = system_instruction::transfer(
        &ctx.accounts.sol_vault.key(),
        &ctx.accounts.trader.key(),
        sol_amount,
    );

    // `sol_vault` là PDA, cần signer seeds
    let pool_key = pool.key(); // Lưu key vào biến
    let seeds: &[&[u8]] = &[
        b"sol_vault",
        pool_key.as_ref(),
        &[ctx.bumps.sol_vault],
    ];
    let signer_seeds = &[seeds]; // Định dạng chuẩn cho invoke_signed

    invoke_signed(
        &transfer_instruction,
        &[
            ctx.accounts.sol_vault.to_account_info(),
            trader.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        signer_seeds, // Dùng PDA làm signer
    )?;

    pool.sol_amount -= sol_amount;
    pool.token_amount += token_amount;

    msg!("Swapped {} tokens for {} SOL", token_amount, sol_amount);
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient token balance")]
    InsufficientTokenBalance,

    #[msg("Insufficient sol balance in pool")]
    InsufficientPoolSolBalance,
}