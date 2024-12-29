use anchor_lang::{account, InitSpace};
use anchor_lang::prelude::*;
use crate::utils::Q64_64;

#[account]
#[derive(InitSpace)]
pub struct CpAmm {
    // Base liquidity that will be locked forever after pool launch
    // Used for stabilizing pool if empty
    initial_locked_base_liquidity: u64, // 8
    // Quote liquidity that will be locked forever after pool launch
    // Used for stabilizing pool if empty
    initial_locked_quote_liquidity: u64, // 8
    
    // Square root of the constant product of the pool
    // Stored as square root in Q64.64 for computation accuracy 
    sqrt_constant_product: Q64_64, // 16
    // Square root of the Base and Quote token's ration
    // Stored as square root in Q64.64 for computation accuracy 
    base_quote_ratio: Q64_64, // 16
    
    // Base token amount in pool's vault
    base_liquidity: u64,   // 8
    // Quote token amount in pool's vault
    quote_liquidity: u64,  // 8
    // Amount of lp tokens minted to liquidity providers
    lp_tokens_supply: u64, // 8
    
    // Fee rate in basis points set by pool creator (1 = 0.01%)
    fee_rate_base_points: u16, // 2
    // Protocol fee from bound AmmsConfig account (1 = 0.01%)
    protocol_fee_rate_base_points: u16, // 2
    
    // Base token fees to redeem by bound AmmsConfig account's authority 
    protocol_base_fees_to_redeem: u64,  // 8
    // Quote token fees to redeem by bound AmmsConfig account's authority 
    protocol_quote_fees_to_redeem: u64, // 8
    
    // Mint of the base token
    base_mint: Pubkey,  // 32
    // Mint of the quote token
    quote_mint: Pubkey, // 32
    // Mint of the liquidity token
    lp_mint: Pubkey,    // 32
    
    // Liquidity vault with base tokens
    base_vault: Pubkey, // 32
    // Liquidity vault with quote tokens
    quote_vault: Pubkey, // 32
    
    // AmmsConfig account
    amms_config: Pubkey, // 32
    // Canonical bump
    bump: u8, // 1
}

impl CpAmm {
    pub const SEED: &'static[u8] = b"cp_amm";
    
    
    
}