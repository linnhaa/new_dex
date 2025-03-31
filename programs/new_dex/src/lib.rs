use anchor_lang::prelude::*;
use crate::instructions::*;

pub mod instructions;
pub mod state;

declare_id!("C6yGxJWKa1d3GyD4Wz4Bi9MXtqPVhme5N1N8Cr6RU4mY");

#[program]
pub mod new_dex {
    use super::*;

    pub fn initialize(
        ctx: Context<InitializePool>,
    ) -> Result<()> {
        instructions::initialize::initialize(ctx)
    }

    pub fn addliquidity(
        ctx: Context<AddLiquidity>,
        sol_amount: u64,
        token_amount: u64
    ) -> Result<()> {
        instructions::addliquidity::add_liquidity(ctx, sol_amount, token_amount)
    }

    pub fn removeliquidity(
        ctx: Context<RemoveLiquidity>,
        sol_amount: u64,
        token_amount: u64
    ) -> Result<()> {
        instructions::removeliquidity::remove_liquidity(ctx, sol_amount, token_amount)
    }

    pub fn swapsol(
        ctx: Context<SwapSol>,
        sol_amount: u64
    ) -> Result<()> {
        instructions::swapsol::swap_sol(ctx, sol_amount)
    }

    pub fn swaptoken(
        ctx: Context<SwapToken>,
        token_amount: u64
    ) -> Result<()> {
        instructions::swaptoken::swap_token(ctx, token_amount)
    }
}
