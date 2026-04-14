mod configs;
mod utils;

use std::sync::Arc;

use crate::configs::{Config, PriceSource};
use crate::utils::{run_event_listener, run_price_updater};
use anyhow::{anyhow, Context, Result};
use dotenvy::dotenv;
use solana_sdk::signature::{read_keypair_file, Signer};

#[tokio::main]
async fn main() -> Result<()> {
  dotenv().ok();
  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();

  let cfg = Config::from_env()?;
  let price_source = PriceSource::from_config(&cfg);
  let admin = Arc::new(
    read_keypair_file(&cfg.backend_keypair_path)
      .map_err(|e| anyhow!(e.to_string()))
      .context("read backend keypair")?,
  );

  let price_task =
    tokio::spawn(run_price_updater(cfg.clone(), price_source, admin.clone()));
  let listener_task = tokio::spawn(run_event_listener(cfg.clone()));

  let (price_res, listener_res) = tokio::try_join!(price_task, listener_task)?;
  price_res?;
  listener_res?;
  Ok(())
}
