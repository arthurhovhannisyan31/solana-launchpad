use crate::OracleState;
use anchor_lang::prelude::*;

#[cfg(feature = "local-testing")]
#[derive(Accounts)]
pub struct ResetOracle<'info> {
  #[account(mut)]
  pub oracle: Account<'info, OracleState>,
}

#[cfg(feature = "local-testing")]
pub fn reset_oracle(ctx: Context<ResetOracle>, new_slot: u64) -> Result<()> {
  let oracle = &mut ctx.accounts.oracle;

  oracle.last_updated_slot = new_slot;

  Ok(())
}
