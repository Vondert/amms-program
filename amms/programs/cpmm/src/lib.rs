use anchor_lang::prelude::*;

declare_id!("EFta43Um9GJB1ABsMh18srSEzsq1EbE3k9uU9aL3dsR3");

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

    pub fn update_amms_config_providers_fee_rate(ctx: Context<UpdateAmmsConfigProvidersFeeRate>, new_providers_fee_rate_basis_points: u16) -> Result<()>{
        update_amms_config_providers_fee_rate::handler(ctx, new_providers_fee_rate_basis_points)
    }

    pub fn update_amms_config_protocol_fee_rate(ctx: Context<UpdateAmmsConfigProtocolFeeRate>, new_protocol_fee_rate_basis_points: u16) -> Result<()>{
        update_amms_config_protocol_fee_rate::handler(ctx, new_protocol_fee_rate_basis_points)
    }

    
    pub fn initialize_cp_amm(ctx: Context<InitializeCpAmm>) -> Result<()>{
        initialize_cp_amm::handler(ctx)
    }
    pub fn launch_cp_amm(ctx: Context<LaunchCpAmm>, base_liquidity: u64, quote_liquidity: u64) -> Result<()>{
        launch_cp_amm::handler(ctx, base_liquidity, quote_liquidity)
    }
    pub fn provide_to_cp_amm(ctx: Context<ProvideToCpAmm>, base_liquidity: u64, quote_liquidity: u64) -> Result<()>{
        provide_to_cp_amm::handler(ctx, base_liquidity, quote_liquidity)
    }
    pub fn withdraw_from_cp_amm(ctx: Context<WithdrawFromCpAmm>, lp_tokens: u64) -> Result<()>{
        withdraw_from_cp_amm::handler(ctx, lp_tokens)
    }
    pub fn swap_in_cp_amm(ctx: Context<SwapInCpAmm>, swap_amount: u64, estimated_result: u64, allowed_slippage: u64, is_in_out: bool) -> Result<()>{
        swap_in_cp_amm::handler(ctx, swap_amount, estimated_result, allowed_slippage, is_in_out)
    }
    pub fn collect_fees_from_cp_amm(ctx: Context<CollectFeesFromCpAmm>) -> Result<()>{
        collect_fees_from_cp_amm::handler(ctx)
    }
}