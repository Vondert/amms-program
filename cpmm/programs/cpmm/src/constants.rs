use anchor_lang::prelude::*;

#[constant]
pub const AMMS_CONFIG_SEED: &[u8] = b"amms_config";
pub const AMMS_CONFIG_MANAGER_SEED: &[u8] = b"amms_configs_manager";

pub const AMMS_CONFIG_MANAGER_INITIALIZE_AUTHORITY_PUBKEY: Pubkey = pubkey!("CmRPa7dPwmzwdpzhCB1YSPh5qoZry6mmExkyMsR23yfF");