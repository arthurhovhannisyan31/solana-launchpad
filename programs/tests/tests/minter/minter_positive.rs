use anchor_lang::prelude::{
  instruction::Instruction, system_program, sysvar::SysvarId, Pubkey, Rent,
};
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use anchor_spl::token::{spl_token::state::AccountState, Mint, TokenAccount};
use anchor_spl::{associated_token, token};
use litesvm::LiteSVM;
use sol_usd_oracle::{self, constants::PRICE_DECIMALS, state::OracleState};
use solana_sdk::{
  signature::{Keypair, Signer},
  transaction::Transaction,
};
use token_minter::state::MinterConfig;
use token_minter::utils::{calc_amount_raw, compute_fee_lamports};

const ORACLE_PRICE: u64 = 100;
const MINT_FEE_USD: u64 = 1000;
const MINT_INITIAL_SUPPLY: u64 = 1_000_000;

#[test]
fn test_minter_positive() -> anyhow::Result<()> {
  let mut svm = LiteSVM::new();
  let oracle_program_id = sol_usd_oracle::ID;
  svm.add_program_from_file(
    oracle_program_id,
    "../../target/deploy/sol_usd_oracle.so",
  )?;
  let minter_program_id = token_minter::ID;
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
  let (mint_config_pda, _bump) =
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

  // 4. Create mint token, create user token account
  let treasury_balance_before = svm.get_balance(&treasury.pubkey()).unwrap();
  let initial_supply: u64 =
    calc_amount_raw(MINT_INITIAL_SUPPLY, oracle_state.decimals)?;

  let mint_token_ix = Instruction {
    program_id: minter_program_id,
    accounts: token_minter::accounts::MintToken {
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
    }
    .to_account_metas(None),
    data: token_minter::instruction::MintToken {
      decimals: PRICE_DECIMALS,
      initial_supply,
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

  svm
    .send_transaction(tx)
    .map_err(|e| anyhow::anyhow!("{:#?}", e))?;

  // Assert mint account created correctly
  let treasury_balance_after = svm.get_balance(&treasury.pubkey()).unwrap();
  let transfer_amount = treasury_balance_after - treasury_balance_before;
  assert_eq!(
    transfer_amount,
    compute_fee_lamports(MINT_FEE_USD, oracle_state.price)?
  );

  let mint_account = svm.get_account(&mint.pubkey()).unwrap();
  assert_eq!(mint_account.owner, token::ID);

  // Assert mint data is correct
  let mint_account = svm.get_account(&mint.pubkey()).unwrap();
  let mint_data = Mint::try_deserialize(&mut &mint_account.data[..])?;

  assert_eq!(mint_data.mint_authority.unwrap(), payer.pubkey());
  assert_eq!(mint_data.freeze_authority.unwrap(), payer.pubkey());
  assert_eq!(
    mint_data.supply,
    calc_amount_raw(MINT_INITIAL_SUPPLY, oracle_state.decimals)?
  );
  assert_eq!(mint_data.decimals, oracle_state.decimals);
  assert_eq!(mint_data.is_initialized, true);

  // Assert user ATA created correctly
  let user_ata_account = svm.get_account(&user_ata).unwrap();
  let user_ata =
    TokenAccount::try_deserialize(&mut &user_ata_account.data[..])?;

  assert_eq!(user_ata.mint, mint.pubkey());
  assert_eq!(user_ata.owner, payer.pubkey());
  assert_eq!(user_ata.state, AccountState::Initialized);
  assert_eq!(user_ata.amount, initial_supply);

  Ok(())
}
