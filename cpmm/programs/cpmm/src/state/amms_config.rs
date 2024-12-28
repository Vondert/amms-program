use anchor_lang::{account, InitSpace};
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct AmmsConfig {
    pub bump: u8,   // 1
    // Id of the config in AmmsConfigsManagers's configs collection
    pub id: u64,    // 8
    // Authority that will receive fees from pool
    pub fee_authority: Pubkey,  // 32
    // In base points from 0 to 10000
    pub fee_rate: u16,  // 2
}

impl AmmsConfig {
    pub fn initialize(&mut self, fee_authority: Pubkey, fee_rate: u16, bump: u8) -> () {
        self.bump = bump;
        self.update_fee_rate(fee_rate);
        self.update_fee_authority(fee_authority);
    }
    pub fn update_fee_authority(&mut self, fee_authority: Pubkey) -> () {
        self.fee_authority = fee_authority;
    }
    pub fn update_fee_rate(&mut self, fee_rate: u16) -> () {
        self.fee_rate = fee_rate;
    }
}