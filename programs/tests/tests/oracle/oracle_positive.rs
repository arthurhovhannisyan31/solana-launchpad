use anchor_lang::prelude::system_program;
use anchor_lang::{
  prelude::Pubkey, AccountDeserialize, InstructionData, ToAccountMetas,
};
use litesvm::LiteSVM;
use sol_usd_oracle::constants::PRICE_DECIMALS;
use sol_usd_oracle::{accounts, instruction, state::OracleState};
use solana_sdk::{
  instruction::Instruction,
  signature::{Keypair, Signer},
  transaction::Transaction,
};

#[test]
fn test_oracle_positive() -> anyhow::Result<()> {
  let mut svm = LiteSVM::new();
  let program_id = sol_usd_oracle::ID;
  svm.add_program_from_file(
    program_id,
    "../../target/deploy/sol_usd_oracle.so",
  )?;
  let payer = Keypair::new();
  svm
    .airdrop(&payer.pubkey(), 1_000_000_000)
    .expect("Failed to airdrop");

  let (oracle_pda, bump) =
    Pubkey::find_program_address(&[OracleState::SEED], &program_id);

  // 1. Initialize Oracle
  let init_ix = Instruction {
    program_id,
    accounts: accounts::InitializeOracle {
      admin: payer.pubkey(),
      oracle: oracle_pda,
      system_program: system_program::ID,
    }
    .to_account_metas(None),
    data: instruction::InitializeOracle {
      admin: payer.pubkey(),
    }
    .data(),
  };

  let tx = Transaction::new_signed_with_payer(
    &[init_ix],
    Some(&payer.pubkey()),
    &[&payer],
    svm.latest_blockhash(),
  );

  svm
    .send_transaction(tx)
    .map_err(|e| anyhow::anyhow!("{:?}", e))?;

  // Check Initial State
  let account = svm.get_account(&oracle_pda).unwrap();
  let state = OracleState::try_deserialize(&mut &account.data[..])?;

  assert_eq!(state.price, 0);
  assert_eq!(state.admin, payer.pubkey());
  assert_eq!(state.decimals, PRICE_DECIMALS);
  assert_eq!(state.bump, bump);

  // 2. Update Price
  let new_price: u64 = 100;
  let update_ix = Instruction {
    program_id,
    accounts: accounts::UpdatePrice {
      admin: payer.pubkey(),
      oracle: oracle_pda,
    }
    .to_account_metas(None),
    data: instruction::UpdatePrice { new_price }.data(),
  };

  let tx = Transaction::new_signed_with_payer(
    &[update_ix],
    Some(&payer.pubkey()),
    &[&payer],
    svm.latest_blockhash(),
  );

  svm
    .send_transaction(tx)
    .map_err(|e| anyhow::anyhow!("{:?}", e))?;

  // Assert Final State
  let account = svm.get_account(&oracle_pda).unwrap();
  let state = OracleState::try_deserialize(&mut &account.data[..])?;
  assert_eq!(state.price, new_price);

  Ok(())
}
