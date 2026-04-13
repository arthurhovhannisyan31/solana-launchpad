use crate::error::MinterError;
use crate::LAMPORTS_PER_SOL_U64;
use anchor_lang::{require, Result};

pub fn compute_fee_lamports(mint_fee_usd: u64, price: u64) -> Result<u64> {
  require!(price > 0, MinterError::OraclePriceZero);

  let fee = mint_fee_usd as u128;
  let lps = LAMPORTS_PER_SOL_U64 as u128;

  let converted_fee = fee.checked_mul(lps).ok_or(MinterError::MathOverflow)?;

  let fee_lamports_u128 = converted_fee
    .checked_div(price as u128)
    .ok_or(MinterError::MathOverflow)?;

  let fee_lamports =
    u64::try_from(fee_lamports_u128).map_err(|_| MinterError::MathOverflow)?;

  Ok(fee_lamports)
}

pub fn calc_amount_raw(initial_supply: u64, decimals: u8) -> Result<u64> {
  let factor = 10u64
    .checked_pow(decimals as u32)
    .ok_or(MinterError::MathOverflow)?;
  let amount_raw = initial_supply
    .checked_mul(factor)
    .ok_or(MinterError::MathOverflow)?;

  Ok(amount_raw)
}
