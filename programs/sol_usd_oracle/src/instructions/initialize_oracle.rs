use anchor_lang::prelude::*;

use crate::{OracleState, DISCRIMINANT, PRICE_DECIMALS};

#[derive(Accounts)]
pub struct InitializeOracle<'info> {
  #[account(
    init,
    payer = admin,
    seeds = [OracleState::SEED, admin.key().as_ref()],
    bump,
    space = DISCRIMINANT + OracleState::SIZE
  )]
  pub oracle: Account<'info, OracleState>,
  #[account(mut)]
  pub admin: Signer<'info>,
  pub system_program: Program<'info, System>,
}

pub fn initialize_oracle(
  ctx: Context<InitializeOracle>,
  admin: Pubkey,
) -> Result<()> {
  let oracle = &mut ctx.accounts.oracle;

  oracle.admin = admin;
  oracle.price = 0;
  oracle.decimals = PRICE_DECIMALS;
  oracle.last_updated_slot = Clock::get()?.slot;
  oracle.bump = ctx.bumps.oracle;

  Ok(())
}
