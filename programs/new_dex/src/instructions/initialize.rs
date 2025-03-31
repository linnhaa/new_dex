use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};

#[account]
pub struct Pool {
    pub authority: Pubkey,
    pub bump: u8,
    pub token_mint: Pubkey,
    pub sol_vault: Pubkey,
    pub token_vault: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(
        init,
        seeds = [b"POOL"],
        payer = authority,
        space = 8 + std::mem::size_of::<Pool>(),
        bump,
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(mut)]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = token_vault.owner == pool.key(),
        constraint = token_vault.mint == token_mint.key()
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"sol_vault", pool.key().as_ref()],
        bump
    )]
    pub sol_vault: SystemAccount<'info>,
    

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}


pub(crate) fn initialize(
    ctx: Context<InitializePool>,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    pool.bump = ctx.bumps.pool;
    pool.authority = ctx.accounts.authority.key();
    pool.token_mint = ctx.accounts.token_mint.key();
    pool.sol_vault = ctx.accounts.sol_vault.key();
    pool.token_vault = ctx.accounts.token_vault.key();
    pool.sol_amount = 0;
    pool.token_amount = 0;
    msg!(
        "Pool is initialized !"
    );
    Ok(())
}


