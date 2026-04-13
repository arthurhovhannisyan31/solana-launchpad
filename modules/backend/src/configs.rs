use std::{env, str::FromStr, time::Duration};

use crate::utils::to_fixed_6;
use anyhow::{Context, Result};
use serde::Serialize;
use solana_sdk::pubkey::Pubkey;

const DEFAULT_PRICE_POLL_INTERVAL_SEC: u64 = 600; // 10 minutes; live price from Binance when MOCK_PRICE is not set

#[derive(Clone)]
pub struct Config {
  pub rpc_http: String,
  pub rpc_ws: String,
  pub oracle_program_id: Pubkey,
  pub oracle_state: Pubkey,
  pub minter_program_id: Pubkey,
  pub backend_keypair_path: String,
  pub price_poll_interval: Duration,
  pub mock_price: Option<u64>,
  pub price_api_url: Option<String>,
}

impl Config {
  pub fn from_env() -> Result<Self> {
    let rpc_http = env::var("SOLANA_RPC_HTTP")
      .context("SOLANA_RPC_HTTP env var is required")?;
    let rpc_ws =
      env::var("SOLANA_RPC_WS").context("SOLANA_RPC_WS env var is required")?;
    let oracle_program_id = Pubkey::from_str(
      &env::var("ORACLE_PROGRAM_ID")
        .context("ORACLE_PROGRAM_ID is required")?,
    )?;
    let oracle_state = Pubkey::from_str(
      &env::var("ORACLE_STATE_PUBKEY").context("ORACLE_STATE_PUBKEY")?,
    )?;
    let minter_program_id = Pubkey::from_str(
      &env::var("MINTER_PROGRAM_ID")
        .context("MINTER_PROGRAM_ID is required")?,
    )?;
    let mut backend_keypair_path = env::var("BACKEND_KEYPAIR_PATH")
      .context("BACKEND_KEYPAIR_PATH is required")?;
    if backend_keypair_path.starts_with("~/") {
      if let Some(home) = env::var_os("HOME") {
        backend_keypair_path =
          format!("{}/{}", home.to_string_lossy(), &backend_keypair_path[2..]);
      }
    }
    let poll = env::var("PRICE_POLL_INTERVAL_SEC")
      .ok()
      .and_then(|s| s.parse::<u64>().ok())
      .unwrap_or(DEFAULT_PRICE_POLL_INTERVAL_SEC);
    let mock_price = env::var("MOCK_PRICE")
      .ok()
      .and_then(|s| s.parse::<u64>().ok());
    let price_api_url = env::var("PRICE_API_URL").ok();

    Ok(Self {
      rpc_http,
      rpc_ws,
      oracle_program_id,
      oracle_state,
      minter_program_id,
      backend_keypair_path,
      price_poll_interval: Duration::from_secs(poll),
      mock_price,
      price_api_url,
    })
  }
}

#[derive(Clone)]
pub enum PriceSource {
  Mock(u64),
  Http { url: String },
}

impl PriceSource {
  pub fn from_config(cfg: &Config) -> Self {
    if let Some(mock) = cfg.mock_price {
      PriceSource::Mock(mock)
    } else if let Some(url) = cfg.price_api_url.clone() {
      PriceSource::Http { url }
    } else {
      PriceSource::Http {
        url: "https://api.binance.com/api/v3/ticker/price?symbol=SOLUSDT"
          .to_string(),
      }
    }
  }

  pub(crate) async fn fetch_price(&self) -> Result<u64> {
    match self {
      PriceSource::Mock(val) => Ok(*val),
      PriceSource::Http { url } => {
        #[derive(serde::Deserialize)]
        struct Resp {
          price: String,
        }
        let resp: Resp = reqwest::get(url).await?.json().await?;
        to_fixed_6(&resp.price)
      }
    }
  }
}

#[derive(Debug, Serialize)]
pub struct TokenCreatedLog {
  pub creator: String,
  pub mint: String,
  pub decimals: u8,
  pub initial_supply: u64,
  pub fee_lamports: u64,
  pub sol_usd_price: u64,
  pub slot: u64,
  pub signature: String,
}
