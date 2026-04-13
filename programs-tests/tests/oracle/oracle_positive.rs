#[cfg(test)]
mod test_oracle_positive {
  use anchor_client::{
    anchor_lang::prelude::Pubkey, anchor_lang::system_program,
    solana_sdk::commitment_config::CommitmentConfig,
    solana_sdk::signature::Signer, Client, Cluster,
  };
  use programs_tests::{sync_airdrop, sync_confirm_transaction};
  use sol_usd_oracle::{
    accounts, constants::PRICE_DECIMALS, instruction, state::OracleState,
  };
  use solana_keypair::Keypair;

  #[test]
  fn test() -> anyhow::Result<()> {
    let program_id = "2qTPCfB4yVywxyVWbsR4LxmgZSschrBVMre5kfeLRSxv";
    // Use random keypair to avoid key conflicts
    let payer = Keypair::new();

    let client = Client::new_with_options(
      Cluster::Localnet,
      &payer,
      CommitmentConfig::confirmed(),
    );
    let program_id = Pubkey::try_from(program_id)?;
    let program = client.program(program_id)?;

    let (oracle_pda, bump) = Pubkey::find_program_address(
      &[OracleState::SEED, &payer.pubkey().as_ref()],
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

    let state: OracleState = program.account(oracle_pda)?;

    // Assert oracle initialization
    assert_eq!(state.price, 0);
    assert_eq!(state.admin, payer.pubkey());
    assert_eq!(state.decimals, PRICE_DECIMALS);
    assert_eq!(state.bump, bump);

    let oracle_account = program.rpc().get_account(&oracle_pda)?;
    assert_eq!(oracle_account.owner, program_id);

    // Update price
    let new_price: u64 = 100;
    program
      .request()
      .accounts(accounts::UpdatePrice {
        admin: payer.pubkey(),
        oracle: oracle_pda,
      })
      .args(instruction::UpdatePrice { new_price })
      .send()?;

    // Assert oracle price update
    let state: OracleState = program.account(oracle_pda)?;
    assert_eq!(state.price, new_price);

    Ok(())
  }
}
