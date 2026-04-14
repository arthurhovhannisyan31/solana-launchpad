#[cfg(test)]
mod test_mint_config_negative {
  use anchor_client::anchor_lang::prelude::system_program;
  use anchor_client::{
    anchor_lang::prelude::Pubkey,
    solana_sdk::commitment_config::CommitmentConfig,
    solana_sdk::signature::Signer, Client, Cluster,
  };
  use programs_tests::{sync_airdrop, sync_confirm_transaction};
  use sol_usd_oracle::{self, state::OracleState};
  use solana_keypair::Keypair;
  use token_minter::state::MinterConfig;

  #[ignore]
  #[test]
  fn test() -> anyhow::Result<()> {
    let oracle_program_id = "24UJLhNSDEwFrziTkshg6Rt18K7H3RczKXR8fNpQ8xa3";
    let minter_program_id = "DXm5uV6Zh3HZshCSUtfoodGDuyDrKnzmP3Nq29PTmYrU";
    // Use random keypair to avoid key conflicts
    let payer = Keypair::new();
    let treasury = Keypair::new();

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

    // Rejects mint config initialize when mint fee is zero
    let res = minter_program
      .request()
      .accounts(token_minter::accounts::InitializeMinter {
        admin: payer.pubkey(),
        config: mint_config_pda,
        system_program: system_program::ID,
      })
      .args(token_minter::instruction::InitializeMinter {
        treasury: treasury.pubkey(),
        mint_fee_usd: 0,
        oracle_state: oracle_pda,
        oracle_program: sol_usd_oracle::ID,
      })
      .send();
    assert!(res.is_err());

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

    // Rejects set new fee equals zero
    let res = minter_program
      .request()
      .accounts(token_minter::accounts::SetFeeUsd {
        admin: payer.pubkey(),
        config: mint_config_pda,
      })
      .args(token_minter::instruction::SetFeeUsd { new_fee_usd: 0 })
      .send();
    assert!(res.is_err());

    Ok(())
  }
}
