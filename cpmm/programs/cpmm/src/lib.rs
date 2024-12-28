use anchor_lang::prelude::*;

declare_id!("6ysJYaHRoUDJomWh3bmpKEdHzEfVEM4LgfQJU2DVRDu3");

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;


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
}
