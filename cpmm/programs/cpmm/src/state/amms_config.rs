use anchor_lang::{account, InitSpace};
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct AmmsConfig {
    pub bump: u8,   // 1
    // Id of the config in AmmsConfigsManagers's configs collection
    pub id: u64,    // 8
    // Authority that will collect fees from pool
    pub fee_authority: Pubkey,  // 32
    // In base points from 0 to 10000
    pub fee_rate_basis_points: u16,  // 2
}

impl AmmsConfig {
    pub const SEED: &'static [u8] = b"amms_config";
    pub fn initialize(&mut self, fee_authority: Pubkey, fee_rate_basis_points: u16, id: u64, bump: u8) -> () {
        self.bump = bump;
        self.id = id;
        self.fee_rate_basis_points = fee_rate_basis_points;
        self.update_fee_authority(fee_authority);
    }
    pub fn update_fee_authority(&mut self, fee_authority: Pubkey) -> () {
        self.fee_authority = fee_authority;
    }
}