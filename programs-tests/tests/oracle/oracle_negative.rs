#[cfg(test)]
mod test_oracle_negative {
  use anchor_client::{
    anchor_lang::prelude::Pubkey, anchor_lang::system_program,
    solana_sdk::commitment_config::CommitmentConfig,
    solana_sdk::signature::Signer, Client, Cluster,
  };
  use programs_tests::{sync_airdrop, sync_confirm_transaction};
  use sol_usd_oracle::constants::PRICE_DECIMALS;
  use sol_usd_oracle::{accounts, instruction, state::OracleState};
  use solana_keypair::Keypair;

  #[test]
  fn test() -> anyhow::Result<()> {
    let program_id = "24UJLhNSDEwFrziTkshg6Rt18K7H3RczKXR8fNpQ8xa3";
    // Use random keypair to avoid key conflicts
    let payer = Keypair::new();
    let fake_payer = Keypair::new();

    let client = Client::new_with_options(
      Cluster::Localnet,
      &payer,
      CommitmentConfig::confirmed(),
    );
    let program_id = Pubkey::try_from(program_id)?;
    let program = client.program(program_id)?;

    let (oracle_pda, _bump) =
      Pubkey::find_program_address(&[OracleState::SEED], &program_id);
    let (danger_pda, _bump) = Pubkey::find_program_address(
      &[OracleState::SEED, b"danger"],
      &program_id,
    );

    sync_airdrop(&program, &payer.pubkey(), 1)?;

    // Initialize_oracle sets admin and defaults
    let signature = program
      .request()
      .accounts(accounts::InitializeOracle {
        admin: payer.pubkey(),
        oracle: oracle_pda,
        system_program: system_program::ID,
      })
      .args(instruction::InitializeOracle {
        admin: payer.pubkey(),
      })
      .send()?;
    sync_confirm_transaction(&program, &signature)?;

    // Update price
    let init_price: u64 = 100;
    program
      .request()
      .accounts(accounts::UpdatePrice {
        admin: payer.pubkey(),
        oracle: oracle_pda,
      })
      .args(instruction::UpdatePrice {
        new_price: init_price,
      })
      .send()?;

    // Unauthorized access
    let new_price: u64 = 0;
    let res = program
      .request()
      .accounts(accounts::UpdatePrice {
        admin: fake_payer.pubkey(),
        oracle: oracle_pda,
      })
      .args(instruction::UpdatePrice { new_price })
      .signer(fake_payer)
      .send();
    assert!(res.is_err());

    // Account spoofing
    let new_price: u64 = 0;
    let res = program
      .request()
      .accounts(accounts::UpdatePrice {
        admin: payer.pubkey(),
        oracle: danger_pda,
      })
      .args(instruction::UpdatePrice { new_price })
      .send();
    assert!(res.is_err());

    // Fail to set zero initial price
    let new_price: u64 = 0;
    let res = program
      .request()
      .accounts(accounts::UpdatePrice {
        admin: payer.pubkey(),
        oracle: oracle_pda,
      })
      .args(instruction::UpdatePrice { new_price })
      .send();
    assert!(res.is_err());

    // Assert oracle state didn't change
    let oracle_state: OracleState = program.account(oracle_pda)?;

    assert_eq!(oracle_state.price, init_price);
    assert_eq!(oracle_state.admin, payer.pubkey());
    assert_eq!(oracle_state.decimals, PRICE_DECIMALS);

    Ok(())
  }
}
