use anchor_lang::prelude::*;

use crate::MinterConfig;

#[derive(Accounts)]
pub struct SetTreasury<'info> {
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

pub fn set_treasury(
  ctx: Context<crate::SetTreasury>,
  new_treasury: Pubkey,
) -> Result<()> {
  ctx.accounts.config.treasury = new_treasury;

  Ok(())
}
