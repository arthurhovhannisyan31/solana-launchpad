use crate::{error::OracleError, state::OracleState};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
  #[account(
    mut,
    seeds = [OracleState::SEED],
    bump = oracle.bump,
    has_one = admin
  )]
  pub oracle: Account<'info, OracleState>,
  pub admin: Signer<'info>,
}

pub fn update_price(ctx: Context<UpdatePrice>, new_price: u64) -> Result<()> {
  require!(new_price > 0, OracleError::InvalidPrice);

  let oracle = &mut ctx.accounts.oracle;
  require_keys_eq!(
    ctx.accounts.admin.key(),
    oracle.admin,
    OracleError::Unauthorized
  );

  let slot = Clock::get()?.slot;

  oracle.price = new_price;
  oracle.last_updated_slot = slot;

  Ok(())
}
