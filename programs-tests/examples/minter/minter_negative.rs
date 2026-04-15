use anchor_client::anchor_lang::prelude::sysvar::SysvarId;
use anchor_client::anchor_lang::prelude::{system_program, Rent};
use anchor_client::{
  anchor_lang::prelude::Pubkey,
  solana_sdk::commitment_config::CommitmentConfig,
  solana_sdk::signature::Signer, Client, Cluster,
};
use anchor_spl::{associated_token, token};
use programs_tests::{sync_airdrop, sync_confirm_transaction};
use sol_usd_oracle::constants::PRICE_DECIMALS;
use sol_usd_oracle::{self, state::OracleState};
use solana_keypair::Keypair;
use token_minter::state::MinterConfig;

fn main() -> anyhow::Result<()> {
  let oracle_program_id = "24UJLhNSDEwFrziTkshg6Rt18K7H3RczKXR8fNpQ8xa3";
  let minter_program_id = "DXm5uV6Zh3HZshCSUtfoodGDuyDrKnzmP3Nq29PTmYrU";
  // Use random keypair to avoid key conflicts
  let payer = Keypair::new();
  let treasury = Keypair::new();
  let mint = Keypair::new();
  let user_ata = associated_token::get_associated_token_address(
    &payer.pubkey(),
    &mint.pubkey(),
  );

  let client = Client::new_with_options(
    Cluster::Localnet,
    &payer,
    CommitmentConfig::confirmed(),
  );
  let oracle_program_id = Pubkey::try_from(oracle_program_id)?;
  let minter_program_id = Pubkey::try_from(minter_program_id)?;
  let oracle_program = client.program(oracle_program_id)?;
  let minter_program = client.program(minter_program_id)?;

  let (oracle_pda, _bump) =
    Pubkey::find_program_address(&[OracleState::SEED], &oracle_program_id);
  let (mint_config_pda, _bump) =
    Pubkey::find_program_address(&[MinterConfig::SEED], &minter_program_id);
  let oracle_price: u64 = 100;
  let mint_fee_usd: u64 = 1000;

  for (pk, multiplier) in [(&payer.pubkey(), 100), (&treasury.pubkey(), 1)] {
    sync_airdrop(&oracle_program, pk, multiplier)?;
  }

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
      new_price: oracle_price,
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
      treasury: treasury.pubkey(),
      mint_fee_usd,
      oracle_state: oracle_pda,
      oracle_program: sol_usd_oracle::ID,
    })
    .send()?;
  sync_confirm_transaction(&minter_program, &signature)?;

  let mint_token_acc = token_minter::accounts::MintToken {
    config: mint_config_pda,
    user: payer.pubkey(),
    treasury: treasury.pubkey(),
    oracle_state: oracle_pda,
    oracle_program: sol_usd_oracle::ID,
    mint: mint.pubkey(),
    user_ata,
    token_program: token::ID,
    associated_token_program: associated_token::ID,
    rent: Rent::id(),
    system_program: system_program::ID,
    token_metadata_program: system_program::ID, // Placeholder account for missing meta program
    metadata: system_program::ID, // Placeholder account for missing meta account
  };

  // Rejects mint creation if supply is zero
  let res = minter_program
    .request()
    .accounts(token_minter::accounts::MintToken { ..mint_token_acc })
    .args(token_minter::instruction::MintToken {
      decimals: PRICE_DECIMALS,
      initial_supply: 0,
      name: "".into(),
      symbol: "".into(),
      uri: "".into(),
    })
    .signer(&mint)
    .send();
  assert!(res.is_err());

  // Rejects mint when decimals exceed allowed range
  let res = minter_program
    .request()
    .accounts(token_minter::accounts::MintToken { ..mint_token_acc })
    .args(token_minter::instruction::MintToken {
      decimals: 10,
      initial_supply: 0,
      name: "".into(),
      symbol: "".into(),
      uri: "".into(),
    })
    .signer(&mint)
    .send();
  assert!(res.is_err());

  Ok(())
}
