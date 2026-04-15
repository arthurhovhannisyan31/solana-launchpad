use anchor_lang::prelude::{system_program, Clock};
use anchor_lang::{
  prelude::Pubkey, AccountDeserialize, InstructionData, ToAccountMetas,
};
use litesvm::LiteSVM;
use sol_usd_oracle::constants::{MAX_STALENESS_SLOTS, PRICE_DECIMALS};
use sol_usd_oracle::{accounts, instruction, state::OracleState};
use solana_sdk::{
  instruction::Instruction,
  signature::{Keypair, Signer},
  transaction::Transaction,
};

const ORACLE_PRICE: u64 = 100;
const BASE_SLOT: u64 = 1000;

#[test]
fn test_oracle_negative() -> anyhow::Result<()> {
  let mut svm = LiteSVM::new();
  let program_id =
    "24UJLhNSDEwFrziTkshg6Rt18K7H3RczKXR8fNpQ8xa3".parse::<Pubkey>()?;
  svm
    .add_program_from_file(program_id, "../target/deploy/sol_usd_oracle.so")?;

  let payer = Keypair::new();
  let fake_payer = Keypair::new();

  for pk in [&payer.pubkey(), &fake_payer.pubkey()] {
    svm.airdrop(pk, 1_000_000_000).expect("Failed to airdrop");
  }

  let (oracle_pda, bump) =
    Pubkey::find_program_address(&[OracleState::SEED], &program_id);
  let (danger_pda, _bump) =
    Pubkey::find_program_address(&[OracleState::SEED, b"danger"], &program_id);

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

  // 2. Update Price
  let update_ix = Instruction {
    program_id,
    accounts: accounts::UpdatePrice {
      admin: payer.pubkey(),
      oracle: oracle_pda,
    }
    .to_account_metas(None),
    data: instruction::UpdatePrice {
      new_price: ORACLE_PRICE,
    }
    .data(),
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

  // 3. Unauthorized access
  let new_price: u64 = 100;
  let update_ix = Instruction {
    program_id,
    accounts: accounts::UpdatePrice {
      admin: fake_payer.pubkey(),
      oracle: oracle_pda,
    }
    .to_account_metas(None),
    data: instruction::UpdatePrice { new_price }.data(),
  };

  let tx = Transaction::new_signed_with_payer(
    &[update_ix],
    Some(&fake_payer.pubkey()),
    &[&fake_payer],
    svm.latest_blockhash(),
  );

  let result = svm.send_transaction(tx);
  assert!(result.is_err());

  // 4. Account spoofing
  let new_price: u64 = 100;
  let update_ix = Instruction {
    program_id,
    accounts: accounts::UpdatePrice {
      admin: payer.pubkey(),
      oracle: danger_pda,
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

  let result = svm.send_transaction(tx);
  assert!(result.is_err());

  // 4. Fail to set zero price
  let new_price: u64 = 0;
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

  let result = svm.send_transaction(tx);
  assert!(result.is_err());

  // 5. Use stale oracle
  // Set base slot since it is 0 by default
  let mut clock = svm.get_sysvar::<Clock>();
  clock.slot = BASE_SLOT;
  svm.set_sysvar::<Clock>(&clock);

  // Reset oracle backward
  let reset_oracle_ix = Instruction {
    program_id,
    accounts: accounts::ResetOracle { oracle: oracle_pda }
      .to_account_metas(None),
    data: instruction::ResetOracle {
      new_slot: clock.slot.saturating_sub(MAX_STALENESS_SLOTS),
    }
    .data(),
  };

  let tx = Transaction::new_signed_with_payer(
    &[reset_oracle_ix],
    Some(&payer.pubkey()),
    &[&payer],
    svm.latest_blockhash(),
  );
  svm
    .send_transaction(tx)
    .map_err(|e| anyhow::anyhow!("{:?}", e))?;

  // Read stale oracle
  let use_oracle_ix = Instruction {
    program_id,
    accounts: accounts::UseOracle { oracle: oracle_pda }.to_account_metas(None),
    data: instruction::UseOracle {}.data(),
  };

  let tx = Transaction::new_signed_with_payer(
    &[use_oracle_ix],
    Some(&payer.pubkey()),
    &[&payer],
    svm.latest_blockhash(),
  );

  let result = svm.send_transaction(tx);
  assert!(result.is_err());

  // 7. Assert Final State
  let account = svm.get_account(&oracle_pda).unwrap();
  let state = OracleState::try_deserialize(&mut &account.data[..])?;

  assert_eq!(state.price, ORACLE_PRICE);
  assert_eq!(state.admin, payer.pubkey());
  assert_eq!(state.decimals, PRICE_DECIMALS);
  assert_eq!(state.bump, bump);

  Ok(())
}
