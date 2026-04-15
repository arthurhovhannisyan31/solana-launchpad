use crate::constants::MAX_STALENESS_SLOTS;
use crate::error::OracleError;
use crate::OracleState;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UseOracle<'info> {
  pub oracle: Account<'info, OracleState>,
}

#[cfg(feature = "local-testing")]
pub fn use_oracle(ctx: Context<UseOracle>) -> Result<()> {
  let oracle = &ctx.accounts.oracle;

  let slot = Clock::get()?.slot;
  let age = slot.saturating_sub(oracle.last_updated_slot);

  require!(age < MAX_STALENESS_SLOTS, OracleError::StaleOracle);

  Ok(())
}
