use crate::constants::ADDRESS_SIZE;
use anchor_lang::prelude::*;

#[account]
#[derive(Debug)]
pub struct OracleState {
  pub admin: Pubkey,
  pub price: u64,
  pub decimals: u8,
  pub last_updated_slot: u64,
  pub bump: u8,
}

impl OracleState {
  pub const SEED: &'static [u8] = b"oracle_state";
  pub const SIZE: usize = ADDRESS_SIZE + 8 + 1 + 8 + 1; // 50 bytes
}
