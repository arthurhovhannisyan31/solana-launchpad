use anchor_lang::prelude::*;

pub mod constants;
mod error;
mod instructions;
pub mod state;
pub mod utils;

use constants::*;
use error::*;
use instructions::*;
use state::*;
use utils::*;

declare_id!("5r5akgcL6qM6kxNjHVgo2qFwjubYuyupmbtKg8zXVEFu");

#[program]
pub mod token_minter {
  use super::*;

  pub fn initialize_minter(
    ctx: Context<InitializeMinter>,
    treasury: Pubkey,
    mint_fee_usd: u64,
    oracle_state: Pubkey,
    oracle_program: Pubkey,
  ) -> Result<()> {
    initialize_minter::initialize_minter(
      ctx,
      treasury,
      mint_fee_usd,
      oracle_state,
      oracle_program,
    )
  }

  pub fn set_fee_usd(ctx: Context<SetFeeUsd>, new_fee_usd: u64) -> Result<()> {
    set_fee_usd::set_fee_usd(ctx, new_fee_usd)
  }

  pub fn set_treasury(
    ctx: Context<SetTreasury>,
    new_treasury: Pubkey,
  ) -> Result<()> {
    set_treasury::set_treasury(ctx, new_treasury)
  }

  pub fn mint_token(
    ctx: Context<MintToken>,
    decimals: u8,
    initial_supply: u64,
    name: String,
    symbol: String,
    uri: String,
  ) -> Result<()> {
    mint_token::mint_token(ctx, decimals, initial_supply, name, symbol, uri)
  }
}
