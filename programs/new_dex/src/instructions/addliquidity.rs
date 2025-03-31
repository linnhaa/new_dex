use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, TokenAccount, Mint};
use anchor_lang::solana_program::system_instruction;
use anchor_lang::solana_program::program::invoke;
use crate::{initialize::Pool,
    state::{LPInfo,
            LP_INFO_SIZE}};

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,

    #[account(mut)]
    pub LP: Signer<'info>,

    #[account(mut)]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = user_token_account.owner == LP.key(),
        constraint = user_token_account.mint == pool.token_mint.key()
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

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

    #[account(
        init_if_needed,
        seeds = [b"lp_info", pool.key().as_ref(), LP.key().as_ref()],
        bump,
        payer = LP,
        space = LP_INFO_SIZE
    )]
    pub lp_info: Box<Account<'info, LPInfo>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn add_liquidity(
    ctx: Context<AddLiquidity>,
    sol_amount: u64,
    token_amount: u64
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    let lp_info = &mut ctx.accounts.lp_info;

    // Nếu là lần đầu add liquidity, khởi tạo thông tin LP
    if lp_info.sol_amount == 0 && lp_info.token_amount == 0 {
        lp_info.owner = ctx.accounts.LP.key();
        lp_info.sol_amount = 0;
        lp_info.token_amount = 0;
    }

    // Transfer SOL to sol_vault using system_program::transfer
    let transfer_instruction = system_instruction::transfer(
        &ctx.accounts.LP.key(),
        &ctx.accounts.sol_vault.key(),
        sol_amount,
    );
    invoke(
        &transfer_instruction,
        &[ 
            ctx.accounts.LP.to_account_info(),
            ctx.accounts.sol_vault.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // Transfer Tokens to token_vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.token_vault.to_account_info(),
        authority: ctx.accounts.LP.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, token_amount)?;

    //Update LP info
    lp_info.sol_amount += sol_amount;
    lp_info.token_amount += token_amount;

    // Update pool state
    pool.sol_amount += sol_amount;
    pool.token_amount += token_amount;

    msg!("Added liquidity: {} SOL, {} Tokens", sol_amount, token_amount);
    Ok(())
}
