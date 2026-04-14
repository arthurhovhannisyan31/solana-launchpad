use anchor_client::anchor_lang::prelude::Pubkey;
use sol_usd_oracle::{self, state::OracleState};
use token_minter::state::MinterConfig;

fn main() -> anyhow::Result<()> {
  let oracle_program_id = "24UJLhNSDEwFrziTkshg6Rt18K7H3RczKXR8fNpQ8xa3";

  let oracle_program_id = Pubkey::try_from(oracle_program_id)?;
  let minter_program_id = Pubkey::try_from(oracle_program_id)?;

  let (oracle_pda, _bump) =
    Pubkey::find_program_address(&[OracleState::SEED], &oracle_program_id);
  let (mint_config_pda, _bump) =
    Pubkey::find_program_address(&[MinterConfig::SEED], &minter_program_id);

  println!("oracle_pda: {oracle_pda}");
  println!("mint_config_pda: {mint_config_pda}");

  Ok(())
}
