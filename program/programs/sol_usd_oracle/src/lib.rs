use anchor_lang::prelude::*;

pub mod constants;
mod error;
mod instructions;
pub mod state;

use constants::*;
use instructions::*;
use state::*;

declare_id!("4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU");

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
}
