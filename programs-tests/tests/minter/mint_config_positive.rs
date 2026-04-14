#[cfg(test)]
mod test_minter_positive {
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

  const INITIAL_PRICE: u64 = 120_000_000;
  const MINT_FEE_USD: u64 = 5_000_000;

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
    let (mint_config_pda, mint_config_bump) =
      Pubkey::find_program_address(&[MinterConfig::SEED], &minter_program_id);
    for (pk, multiplier) in [(&payer.pubkey(), 1), (&treasury.pubkey(), 1)] {
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
        treasury: treasury.pubkey(),
        mint_fee_usd: MINT_FEE_USD,
        oracle_state: oracle_pda,
        oracle_program: sol_usd_oracle::ID,
      })
      .send()?;
    sync_confirm_transaction(&minter_program, &signature)?;

    // Mint config set new fee
    let new_fee_usd: u64 = 10;
    let signature = minter_program
      .request()
      .accounts(token_minter::accounts::SetFeeUsd {
        admin: payer.pubkey(),
        config: mint_config_pda,
      })
      .args(token_minter::instruction::SetFeeUsd { new_fee_usd })
      .send()?;
    sync_confirm_transaction(&minter_program, &signature)?;

    let minter_config: MinterConfig =
      minter_program.account(mint_config_pda)?;

    // Assert mint_fee_usd updated in mint config
    assert_eq!(minter_config.mint_fee_usd, new_fee_usd);

    // Assert rest of the min config fields stay intact
    assert_eq!(minter_config.treasury, treasury.pubkey());
    assert_eq!(minter_config.admin, payer.pubkey());
    assert_eq!(minter_config.oracle_program, sol_usd_oracle::ID);
    assert_eq!(minter_config.oracle_state, oracle_pda);
    assert_eq!(minter_config.bump, mint_config_bump);

    // Mint config set new treasury
    let new_treasury = Keypair::new();
    let signature = minter_program
      .request()
      .accounts(token_minter::accounts::SetTreasury {
        admin: payer.pubkey(),
        config: mint_config_pda,
      })
      .args(token_minter::instruction::SetTreasury {
        new_treasury: new_treasury.pubkey(),
      })
      .send()?;
    sync_confirm_transaction(&minter_program, &signature)?;

    let minter_config: MinterConfig =
      minter_program.account(mint_config_pda)?;

    // Assert new_treasury updated in mint config
    assert_eq!(minter_config.treasury, new_treasury.pubkey());

    // Assert rest of the min config fields stay intact
    assert_eq!(minter_config.mint_fee_usd, new_fee_usd);
    assert_eq!(minter_config.admin, payer.pubkey());
    assert_eq!(minter_config.oracle_program, sol_usd_oracle::ID);
    assert_eq!(minter_config.oracle_state, oracle_pda);
    assert_eq!(minter_config.bump, mint_config_bump);

    Ok(())
  }
}
