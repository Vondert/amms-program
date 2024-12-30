use anchor_lang::{account, InitSpace};
use anchor_lang::prelude::*;
use crate::error::ErrorCode;
#[account]
#[derive(InitSpace)]
pub struct AmmsConfig {
    pub bump: u8,   // 1
    // Id of the config in AmmsConfigsManagers's configs collection
    pub id: u64,    // 8
    // Authority that will collect fees from pool
    pub fee_authority: Pubkey,  // 32
    // Providers fee rate in basis points (1 = 0.01%)
    pub providers_fee_rate_basis_points: u16, // 2
    // Protocol fee rate in basis points (1 = 0.01%)
    pub protocol_fee_rate_basis_points: u16, // 2
}

impl AmmsConfig {
    pub const SEED: &'static [u8] = b"amms_config";
    pub fn initialize(&mut self, fee_authority: Pubkey, protocol_fee_rate_basis_points: u16, providers_fee_rate_basis_points: u16, id: u64, bump: u8) -> Result<()> {
        require!(providers_fee_rate_basis_points + protocol_fee_rate_basis_points <= 10000, ErrorCode::ConfigFeeRateExceeded);
        
        self.bump = bump;
        self.id = id;
        self.protocol_fee_rate_basis_points = protocol_fee_rate_basis_points;
        self.providers_fee_rate_basis_points = providers_fee_rate_basis_points;
        self.update_fee_authority(fee_authority);
        
        Ok(())
    }
    pub fn update_fee_authority(&mut self, fee_authority: Pubkey) -> () {
        self.fee_authority = fee_authority;
    }
}