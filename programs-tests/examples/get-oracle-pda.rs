use anchor_client::{
  anchor_lang::prelude::Pubkey, solana_sdk::signature::Signer,
};
use sol_usd_oracle::{self, state::OracleState};
use solana_keypair::read_keypair_file;

fn main() -> anyhow::Result<()> {
  let oracle_program_id = "24UJLhNSDEwFrziTkshg6Rt18K7H3RczKXR8fNpQ8xa3";
  let payer = read_keypair_file("/home/q/.config/solana/id.json").unwrap();

  let oracle_program_id = Pubkey::try_from(oracle_program_id)?;

  let (oracle_pda, _bump) = Pubkey::find_program_address(
    &[OracleState::SEED, &payer.pubkey().as_ref()],
    &oracle_program_id,
  );

  println!("oracle_pda: {oracle_pda}");

  Ok(())
}
