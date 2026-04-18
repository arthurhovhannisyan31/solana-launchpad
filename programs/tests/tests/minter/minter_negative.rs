use anchor_lang::prelude::{
  instruction::Instruction, system_program, sysvar::SysvarId, Pubkey, Rent,
};
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use anchor_spl::{associated_token, token};
use litesvm::LiteSVM;
use sol_usd_oracle::{self, constants::PRICE_DECIMALS, state::OracleState};
use solana_sdk::{
  signature::{Keypair, Signer},
  transaction::Transaction,
};
use token_minter::state::MinterConfig;
use token_minter::utils::calc_amount_raw;

const ORACLE_PRICE: u64 = 100;
const MINT_FEE_USD: u64 = 1000;
const MINT_INITIAL_SUPPLY: u64 = 1_000_000;

#[test]
fn test_minter_positive() -> anyhow::Result<()> {
  let mut svm = LiteSVM::new();

  let oracle_program_id =
    "24UJLhNSDEwFrziTkshg6Rt18K7H3RczKXR8fNpQ8xa3".parse::<Pubkey>()?;
  svm.add_program_from_file(
    oracle_program_id,
    "../../target/deploy/sol_usd_oracle.so",
  )?;

  let minter_program_id =
    "DXm5uV6Zh3HZshCSUtfoodGDuyDrKnzmP3Nq29PTmYrU".parse::<Pubkey>()?;
  svm.add_program_from_file(
    minter_program_id,
    "../../target/deploy/token_minter.so",
  )?;

  let payer = Keypair::new();
  let treasury = Keypair::new();
  let mint = Keypair::new();
  let user_ata = associated_token::get_associated_token_address(
    &payer.pubkey(),
    &mint.pubkey(),
  );
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

  let account = svm.get_account(&oracle_pda).unwrap();
  let oracle_state = OracleState::try_deserialize(&mut &account.data[..])?;

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

  // 4. Rejects mint creation if supply is zero
  let treasury_balance_before = svm.get_balance(&treasury.pubkey()).unwrap();
  let initial_supply: u64 =
    calc_amount_raw(MINT_INITIAL_SUPPLY, oracle_state.decimals)?;

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

  let mint_token_ix = Instruction {
    program_id: minter_program_id,
    accounts: token_minter::accounts::MintToken { ..mint_token_acc }
      .to_account_metas(None),
    data: token_minter::instruction::MintToken {
      decimals: PRICE_DECIMALS,
      initial_supply: 0,
      name: "".into(),
      symbol: "".into(),
      uri: "".into(),
    }
    .data(),
  };

  let tx = Transaction::new_signed_with_payer(
    &[mint_token_ix],
    Some(&payer.pubkey()),
    &[&payer, &mint],
    svm.latest_blockhash(),
  );

  let res = svm.send_transaction(tx);
  assert!(res.is_err());

  // 5. Rejects mint when decimals exceed allowed range
  let mint_token_ix = Instruction {
    program_id: minter_program_id,
    accounts: token_minter::accounts::MintToken { ..mint_token_acc }
      .to_account_metas(None),
    data: token_minter::instruction::MintToken {
      decimals: 10,
      initial_supply: 0,
      name: "".into(),
      symbol: "".into(),
      uri: "".into(),
    }
    .data(),
  };

  let tx = Transaction::new_signed_with_payer(
    &[mint_token_ix],
    Some(&payer.pubkey()),
    &[&payer, &mint],
    svm.latest_blockhash(),
  );

  let res = svm.send_transaction(tx);
  assert!(res.is_err());

  Ok(())
}
