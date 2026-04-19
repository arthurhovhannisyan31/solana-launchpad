use anchor_client::anchor_lang::prelude::system_program;
use anchor_client::{
  anchor_lang::prelude::Pubkey,
  solana_sdk::commitment_config::CommitmentConfig,
  solana_sdk::signature::Signer, Client, Cluster,
};
use sol_usd_oracle::{self, state::OracleState};
use solana_keypair::read_keypair_file;
use std::str::FromStr;
use tests::sync_confirm_transaction;
use token_minter::state::MinterConfig;

const INITIAL_PRICE: u64 = 120_000_000;
const MINT_FEE_USD: u64 = 5_000_000;

fn main() -> anyhow::Result<()> {
  let oracle_program_id = sol_usd_oracle::ID;
  let minter_program_id = token_minter::ID;
  let payer = read_keypair_file("/home/q/.config/solana/id.json").unwrap();

  let cluster_type = std::env::var("CLUSTER").unwrap_or("localnet".into());
  let cluster = Cluster::from_str(&cluster_type)?;

  let client =
    Client::new_with_options(cluster, &payer, CommitmentConfig::confirmed());
  let oracle_program_id = Pubkey::try_from(oracle_program_id)?;
  let minter_program_id = Pubkey::try_from(minter_program_id)?;
  let oracle_program = client.program(oracle_program_id)?;
  let minter_program = client.program(minter_program_id)?;

  let (oracle_pda, _bump) =
    Pubkey::find_program_address(&[OracleState::SEED], &oracle_program_id);
  let (mint_config_pda, mint_config_bump) =
    Pubkey::find_program_address(&[MinterConfig::SEED], &minter_program_id);

  // Initialize oracle
  let signature = oracle_program
    .request()
    .accounts(sol_usd_oracle::accounts::InitializeOracle {
      admin: payer.pubkey(),
      oracle: oracle_pda,
      system_program: system_program::ID,
    })
    .args(sol_usd_oracle::instruction::InitializeOracle {
      admin: payer.pubkey(),
    })
    .send()?;
  sync_confirm_transaction(&oracle_program, &signature)?;

  // Update oracle price
  oracle_program
    .request()
    .accounts(sol_usd_oracle::accounts::UpdatePrice {
      admin: payer.pubkey(),
      oracle: oracle_pda,
    })
    .args(sol_usd_oracle::instruction::UpdatePrice {
      new_price: INITIAL_PRICE,
    })
    .send()?;

  // Initialize mint config account
  let signature = minter_program
    .request()
    .accounts(token_minter::accounts::InitializeMinter {
      admin: payer.pubkey(),
      config: mint_config_pda,
      system_program: system_program::ID,
    })
    .args(token_minter::instruction::InitializeMinter {
      treasury: payer.pubkey(),
      mint_fee_usd: MINT_FEE_USD,
      oracle_state: oracle_pda,
      oracle_program: sol_usd_oracle::ID,
    })
    .send()?;
  sync_confirm_transaction(&minter_program, &signature)?;

  Ok(())
}
