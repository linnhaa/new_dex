use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, TokenAccount, Mint};
use anchor_lang::solana_program::system_instruction;
use anchor_lang::solana_program::program::invoke;
use crate::initialize::Pool;

#[derive(Accounts)]
pub struct SwapSol<'info> {
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

pub fn swap_sol(
    ctx: Context<SwapSol>,
    sol_amount: u64,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    let sol_balance = ctx.accounts.sol_vault.lamports();
    let token_balance = ctx.accounts.token_vault.amount;
    let k = sol_balance as u128 * token_balance as u128;

    // Calculate token amount using AMM formula
    let new_sol_balance = sol_balance as u128 + sol_amount as u128;
    let new_token_balance = k / new_sol_balance;
    let token_amount = token_balance - new_token_balance as u64;

    // Ensure pool has enough tokens for the swap
    require!(token_balance >= token_amount, ErrorCode::InsufficientPoolTokenBalance);
    require!(ctx.accounts.trader.lamports() >= sol_amount, ErrorCode::InsufficientSolBalance);

    // chuyển sol từ trader sang pool
    let transfer_instruction = system_instruction::transfer(
        &ctx.accounts.trader.key(),
        &ctx.accounts.sol_vault.key(),
        sol_amount,
    );
    invoke(
        &transfer_instruction,
        &[ 
            ctx.accounts.trader.to_account_info(),
            ctx.accounts.sol_vault.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // Transfer Tokens from token_vault back to user_token_account
    let pool_bump = pool.bump; // Lấy bump từ account Pool
    let pool_seeds = &[
        b"POOL",
        &[pool_bump][..],
    ];
    let signer = &[&pool_seeds[..]];

    // Tạo CpiContext với signer
    let cpi_accounts = Transfer {
        from: ctx.accounts.token_vault.to_account_info(),
        to: ctx.accounts.trader_token_account.to_account_info(),
        authority: pool.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer
    );

    // Thực hiện chuyển token
    token::transfer(cpi_ctx, token_amount)?;

    pool.sol_amount += sol_amount;
    pool.token_amount -= token_amount;

    msg!("Swapped {} SOL for {} tokens", sol_amount, token_amount);
    Ok(())


}


#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient SOL balance")]
    InsufficientSolBalance,

    #[msg("Insufficient token balance in pool")]
    InsufficientPoolTokenBalance,
}