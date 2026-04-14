use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use constants::*;
use instructions::*;
use state::*;

declare_id!("24UJLhNSDEwFrziTkshg6Rt18K7H3RczKXR8fNpQ8xa3");

#[program]
pub mod sol_usd_oracle {
  use super::*;

  pub fn initialize_oracle(
    ctx: Context<InitializeOracle>,
    admin: Pubkey,
  ) -> Result<()> {
    initialize_oracle::initialize_oracle(ctx, admin)
  }

  pub fn update_price(ctx: Context<UpdatePrice>, new_price: u64) -> Result<()> {
    update_price::update_price(ctx, new_price)
  }

  #[cfg(feature = "local-testing")]
  pub fn use_oracle(ctx: Context<UseOracle>) -> Result<()> {
    use_oracle::use_oracle(ctx)
  }

  #[cfg(feature = "local-testing")]
  pub fn reset_oracle(ctx: Context<ResetOracle>, new_slot: u64) -> Result<()> {
    reset_oracle::reset_oracle(ctx, new_slot)
  }
}
