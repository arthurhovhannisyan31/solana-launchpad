use crate::error::MinterError;
use crate::LAMPORTS_PER_SOL_U64;
use anchor_lang::require;

pub fn compute_fee_lamports(
  mint_fee_usd: u64,
  price: u64,
) -> anchor_lang::Result<u64> {
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
