use crate::constants::ADDRESS_SIZE;
use anchor_lang::prelude::*;

#[account]
#[derive(Debug, InitSpace)]
pub struct MinterConfig {
  pub admin: Pubkey,
  pub treasury: Pubkey,
  pub mint_fee_usd: u64,
  pub oracle_program: Pubkey,
  pub oracle_state: Pubkey,
  pub bump: u8,
}

impl MinterConfig {
  pub const SEED: &'static [u8] = b"minter_config";
  pub const SIZE: usize =
    ADDRESS_SIZE + ADDRESS_SIZE + 8 + ADDRESS_SIZE + ADDRESS_SIZE + 1;
}

#[event]
pub struct TokenCreated {
  pub creator: Pubkey,
  pub mint: Pubkey,
  pub decimals: u8,
  pub initial_supply: u64,
  pub fee_lamports: u64,
  pub sol_usd_price: u64,
  pub slot: u64,
}
