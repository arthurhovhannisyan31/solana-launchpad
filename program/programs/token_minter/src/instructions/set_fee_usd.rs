use anchor_lang::prelude::*;

use crate::{MinterConfig, MinterError};

#[derive(Accounts)]
pub struct SetFeeUsd<'info> {
  #[account(
    mut,
    seeds = [MinterConfig::SEED],
    bump = config.bump,
    has_one = admin
  )]
  pub config: Account<'info, MinterConfig>,
  // TODO add signer to account seeds to make it easier to test
  pub admin: Signer<'info>,
}

pub fn set_fee_usd(ctx: Context<SetFeeUsd>, new_fee_usd: u64) -> Result<()> {
  require!(new_fee_usd > 0, MinterError::InvalidFeeUsd);

  ctx.accounts.config.mint_fee_usd = new_fee_usd;

  Ok(())
}
