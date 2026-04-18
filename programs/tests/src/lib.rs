use anchor_client::solana_sdk::native_token::LAMPORTS_PER_SOL;
use anchor_client::solana_sdk::signature::Signature;
use anchor_client::Program;
use anyhow::anyhow;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use std::thread;
use std::time::Duration;

pub fn sync_confirm_transaction(
  program: &Program<&Keypair>,
  signature: &Signature,
) -> anyhow::Result<()> {
  loop {
    if program.rpc().confirm_transaction(signature)? {
      return Ok(());
    }
    thread::sleep(Duration::from_millis(50));
  }
}

pub fn sync_airdrop(
  program: &Program<&Keypair>,
  pub_key: &Pubkey,
  sol_multiplier: u64,
) -> anyhow::Result<()> {
  let amount = sol_multiplier
    .checked_mul(LAMPORTS_PER_SOL)
    .ok_or(anyhow!("MathOverflow"))?;

  let signature = program.rpc().request_airdrop(pub_key, amount)?;
  sync_confirm_transaction(program, &signature)
}
