use anchor_lang::prelude::*;

pub const LP_INFO_SIZE: usize = 8 + 32 + 8 + 8 + 1; // 8 bytes discriminator + 32 bytes owner + 8 bytes SOL + 8 bytes Token + 1 byte bump

#[account]
pub struct LPInfo {
    pub owner: Pubkey,       // LP's wallet address
    pub sol_amount: u64,     // Số lượng SOL mà LP đã cung cấp
    pub token_amount: u64,   // Số lượng token mà LP đã cung cấp
    pub bump: u8,            // Bump của PDA
}

