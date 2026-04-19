use anchor_lang::prelude::*;

use crate::{error::MinterError, MinterConfig, DISCRIMINANT};

#[derive(Accounts)]
pub struct InitializeMinter<'info> {
  #[account(
    init,
    payer = admin,
    seeds = [MinterConfig::SEED],
    bump,
    space = DISCRIMINANT + MinterConfig::INIT_SPACE,
  )]
  pub config: Account<'info, MinterConfig>,
  #[account(mut)]
  pub admin: Signer<'info>,
  pub system_program: Program<'info, System>,
}

pub fn initialize_minter(
  ctx: Context<InitializeMinter>,
  treasury: Pubkey,
  mint_fee_usd: u64,
  oracle_state: Pubkey,
  oracle_program: Pubkey,
) -> Result<()> {
  require!(mint_fee_usd > 0, MinterError::InvalidFeeUsd);

  let config = &mut ctx.accounts.config;

  config.admin = ctx.accounts.admin.key();
  config.treasury = treasury;
  config.mint_fee_usd = mint_fee_usd;
  config.oracle_program = oracle_program;
  config.oracle_state = oracle_state;
  config.bump = ctx.bumps.config;

  Ok(())
}
