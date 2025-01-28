use anchor_lang::prelude::*;

#[constant]
pub const AMMS_CONFIG_MANAGER_INITIALIZE_AUTHORITY_PUBKEY: Pubkey = pubkey!("CmRPa7dPwmzwdpzhCB1YSPh5qoZry6mmExkyMsR23yfF");

pub const ANCHOR_DISCRIMINATOR: usize = 8;

pub const CP_AMM_INITIALIZE_PRICE_IN_LAMPORTS: u64 = 100_000_000;