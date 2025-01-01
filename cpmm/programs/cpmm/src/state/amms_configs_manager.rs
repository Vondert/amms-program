use anchor_lang::{account, InitSpace};
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct AmmsConfigsManager {
    pub authority: Pubkey,  // 32
    pub head_authority: Pubkey, // 32
    pub configs_count: u64, // 8
    pub bump: u8,   // 1
}

impl AmmsConfigsManager {
    pub const SEED: &'static [u8] = b"amms_configs_manager";
    pub(crate) fn initialize(&mut self, authority: Pubkey, head_authority: Pubkey, bump: u8) -> () {
        self.bump = bump;
        self.configs_count = 0;
        self.update_authority(authority);
        self.update_head_authority(head_authority);
    }

    pub(crate) fn update_authority(&mut self, authority: Pubkey) {
        self.authority = authority;
    }
    pub(crate) fn update_head_authority(&mut self, head_authority: Pubkey) {
        self.head_authority = head_authority;
    }
    pub(crate) fn increment_configs_count(&mut self) -> () {
        self.configs_count = self.configs_count.checked_add(1).unwrap()
    }
}