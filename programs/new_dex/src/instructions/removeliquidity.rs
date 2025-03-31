use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, TokenAccount, Mint};
use crate::{initialize::Pool, state::{LPInfo, LP_INFO_SIZE}};
use anchor_lang::solana_program::system_instruction;
use anchor_lang::solana_program::program::invoke_signed;



#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(
        mut,
        seeds = [b"POOL"],
        bump,
    )]
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
        mut,
        seeds = [b"lp_info", pool.key().as_ref(), LP.key().as_ref()],
        bump,
    )]
    pub lp_info: Box<Account<'info, LPInfo>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn remove_liquidity(
    ctx: Context<RemoveLiquidity>,
    sol_amount: u64,
    token_amount: u64
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let lp_info = &mut ctx.accounts.lp_info;

    require!(lp_info.sol_amount >= sol_amount, ErrorCode::InsufficientSolLiquidity);
    require!(lp_info.token_amount >= token_amount, ErrorCode::InsufficientTokenLiquidity);

    // Transfer SOL from sol_vault back to LP
    let transfer_instruction = system_instruction::transfer(
        &ctx.accounts.sol_vault.key(),
        &ctx.accounts.LP.key(),
        sol_amount,
    );
    
    // vì `sol_vault` là PDA, cần signer seeds
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
            ctx.accounts.LP.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        signer_seeds, // Dùng PDA làm signer
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
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: pool.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer
    );

    // Thực hiện chuyển token
    token::transfer(cpi_ctx, token_amount)?;

    // Update LP info
    lp_info.sol_amount -= sol_amount;
    lp_info.token_amount -= token_amount;

    // Update pool state
    pool.sol_amount -= sol_amount;
    pool.token_amount -= token_amount;

    msg!("Removed liquidity: {} SOL, {} Tokens", sol_amount, token_amount);
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient SOL liquidity")] 
    InsufficientSolLiquidity,
    
    #[msg("Insufficient Token liquidity")] 
    InsufficientTokenLiquidity,
}
