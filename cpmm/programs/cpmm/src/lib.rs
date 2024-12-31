use anchor_lang::prelude::*;

declare_id!("2wFPV42nma7Lv8fqfUEtzBYGoiZybowLxFokaGGMqGCg");

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

pub use instructions::*;

#[program]
pub mod cpmm {
    use super::*;

    pub fn initialize_amms_configs_manager(ctx: Context<InitializeAmmsConfigsManager>) -> Result<()>{
        initialize_amms_configs_manager::handler(ctx)
    }

    pub fn update_amms_configs_manager_authority(ctx: Context<UpdateAmmsConfigsManagerAuthority>) -> Result<()>{
        update_amms_configs_manager_authority::handler(ctx)
    }

    pub fn update_amms_configs_manager_head_authority(ctx: Context<UpdateAmmsConfigsManagerHeadAuthority>) -> Result<()>{
        update_amms_configs_manager_head_authority::handler(ctx)
    }


    pub fn initialize_amms_config(ctx: Context<InitializeAmmsConfig>, protocol_fee_rate_basis_points: u16, providers_fee_rate_basis_points: u16) -> Result<()>{
        initialize_amms_config::handler(ctx, protocol_fee_rate_basis_points, providers_fee_rate_basis_points)
    }

    pub fn update_amms_config_fee_authority(ctx: Context<UpdateAmmsConfigFeeAuthority>) -> Result<()>{
        update_amms_config_fee_authority::handler(ctx)
    }
    
    
    pub fn initialize_cp_amm(ctx: Context<InitializeCpAmm>) -> Result<()>{
        initialize_cp_amm::handler(ctx)
    }
    pub fn launch_cp_amm(ctx: Context<LaunchCpAmm>, base_liquidity: u64, quote_liquidity: u64) -> Result<()>{
        launch_cp_amm::handler(ctx, base_liquidity, quote_liquidity)
    }
}