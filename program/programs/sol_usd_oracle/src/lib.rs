use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use constants::*;
use instructions::*;
use state::*;

declare_id!("2qd4AXtJFcBB7cLtKH8cPwNEdDqL6PLqQ8dgDWCCxt8U");

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
