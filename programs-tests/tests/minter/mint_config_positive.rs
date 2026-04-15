use anchor_lang::prelude::{instruction::Instruction, system_program, Pubkey};
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use sol_usd_oracle::{self, state::OracleState};
use solana_sdk::{
  signature::{Keypair, Signer},
  transaction::Transaction,
};
use token_minter::state::MinterConfig;

const ORACLE_PRICE: u64 = 100;
const MINT_FEE_USD: u64 = 1000;

#[test]
fn test_mint_config_positive() -> anyhow::Result<()> {
  let mut svm = LiteSVM::new();

  let oracle_program_id =
    "24UJLhNSDEwFrziTkshg6Rt18K7H3RczKXR8fNpQ8xa3".parse::<Pubkey>()?;
  svm.add_program_from_file(
    oracle_program_id,
    "../target/deploy/sol_usd_oracle.so",
  )?;

  let minter_program_id =
    "DXm5uV6Zh3HZshCSUtfoodGDuyDrKnzmP3Nq29PTmYrU".parse::<Pubkey>()?;
  svm.add_program_from_file(
    minter_program_id,
    "../target/deploy/token_minter.so",
  )?;

  let payer = Keypair::new();
  let treasury = Keypair::new();
  let (oracle_pda, _bump) =
    Pubkey::find_program_address(&[OracleState::SEED], &oracle_program_id);
  let (mint_config_pda, mint_config_bump) =
    Pubkey::find_program_address(&[MinterConfig::SEED], &minter_program_id);

  for (pk, multiplier) in [(&payer.pubkey(), 100), (&treasury.pubkey(), 1)] {
    svm
      .airdrop(pk, multiplier * 1_000_000_000)
      .expect("Failed to airdrop");
  }

  // 1. Initialize Oracle
  let init_ix = Instruction {
    program_id: oracle_program_id,
    accounts: sol_usd_oracle::accounts::InitializeOracle {
      admin: payer.pubkey(),
      oracle: oracle_pda,
      system_program: system_program::ID,
    }
    .to_account_metas(None),
    data: sol_usd_oracle::instruction::InitializeOracle {
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
    .map_err(|e| anyhow::anyhow!("{:#?}", e))?;

  // 2. Update Price
  let update_ix = Instruction {
    program_id: oracle_program_id,
    accounts: sol_usd_oracle::accounts::UpdatePrice {
      admin: payer.pubkey(),
      oracle: oracle_pda,
    }
    .to_account_metas(None),
    data: sol_usd_oracle::instruction::UpdatePrice {
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
    .map_err(|e| anyhow::anyhow!("{:#?}", e))?;

  // 3. Initialize mint config account
  let init_minter_ix = Instruction {
    program_id: minter_program_id,
    accounts: token_minter::accounts::InitializeMinter {
      admin: payer.pubkey(),
      config: mint_config_pda,
      system_program: system_program::ID,
    }
    .to_account_metas(None),
    data: token_minter::instruction::InitializeMinter {
      treasury: treasury.pubkey(),
      mint_fee_usd: MINT_FEE_USD,
      oracle_state: oracle_pda,
      oracle_program: sol_usd_oracle::ID,
    }
    .data(),
  };

  let tx = Transaction::new_signed_with_payer(
    &[init_minter_ix],
    Some(&payer.pubkey()),
    &[&payer],
    svm.latest_blockhash(),
  );

  svm
    .send_transaction(tx)
    .map_err(|e| anyhow::anyhow!("{:#?}", e))?;

  let account = svm.get_account(&mint_config_pda).unwrap();
  let minter_config = MinterConfig::try_deserialize(&mut &account.data[..])?;

  assert_eq!(minter_config.treasury, treasury.pubkey());
  assert_eq!(minter_config.admin, payer.pubkey());
  assert_eq!(minter_config.mint_fee_usd, MINT_FEE_USD);
  assert_eq!(minter_config.oracle_program, sol_usd_oracle::ID);
  assert_eq!(minter_config.oracle_state, oracle_pda);
  assert_eq!(minter_config.bump, mint_config_bump);

  // 4. Mint config set new fee
  let new_fee_usd: u64 = 10;
  let set_fee_usd_ix = Instruction {
    program_id: minter_program_id,
    accounts: token_minter::accounts::SetFeeUsd {
      admin: payer.pubkey(),
      config: mint_config_pda,
    }
    .to_account_metas(None),
    data: token_minter::instruction::SetFeeUsd { new_fee_usd }.data(),
  };

  let tx = Transaction::new_signed_with_payer(
    &[set_fee_usd_ix],
    Some(&payer.pubkey()),
    &[&payer],
    svm.latest_blockhash(),
  );

  svm
    .send_transaction(tx)
    .map_err(|e| anyhow::anyhow!("{:#?}", e))?;

  let account = svm.get_account(&mint_config_pda).unwrap();
  let minter_config = MinterConfig::try_deserialize(&mut &account.data[..])?;

  // Assert mint_fee_usd updated in mint config
  assert_eq!(minter_config.mint_fee_usd, new_fee_usd);

  // Assert rest of the min config fields stay intact
  assert_eq!(minter_config.treasury, treasury.pubkey());
  assert_eq!(minter_config.admin, payer.pubkey());
  assert_eq!(minter_config.oracle_program, sol_usd_oracle::ID);
  assert_eq!(minter_config.oracle_state, oracle_pda);
  assert_eq!(minter_config.bump, mint_config_bump);

  // 5. Mint config set new treasury
  let new_treasury = Keypair::new();
  let set_treasury_ix = Instruction {
    program_id: minter_program_id,
    accounts: token_minter::accounts::SetTreasury {
      admin: payer.pubkey(),
      config: mint_config_pda,
    }
    .to_account_metas(None),
    data: token_minter::instruction::SetTreasury {
      new_treasury: new_treasury.pubkey(),
    }
    .data(),
  };

  let tx = Transaction::new_signed_with_payer(
    &[set_treasury_ix],
    Some(&payer.pubkey()),
    &[&payer],
    svm.latest_blockhash(),
  );

  svm
    .send_transaction(tx)
    .map_err(|e| anyhow::anyhow!("{:#?}", e))?;

  let account = svm.get_account(&mint_config_pda).unwrap();
  let minter_config = MinterConfig::try_deserialize(&mut &account.data[..])?;

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
