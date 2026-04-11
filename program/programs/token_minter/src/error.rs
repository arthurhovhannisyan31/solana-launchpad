use anchor_lang::error_code;

#[error_code]
pub enum MinterError {
  #[msg("Mint fee in USD must be greater than zero")]
  InvalidFeeUsd,
  #[msg("Unauthorized admin call")]
  Unauthorized,
  #[msg("Oracle price must be greater than zero")]
  OraclePriceZero,
  #[msg("Math overflow while computing fee")]
  MathOverflow,
  #[msg("Invalid supply value")]
  InvalidSupply,
  #[msg("Decimals out of allowed range")]
  InvalidDecimals,
  #[msg("Oracle account does not match config")]
  InvalidOracleState,
  #[msg("Oracle program does not match config")]
  InvalidOracleProgram,
  #[msg("Oracle decimals mismatch expected 6")]
  OracleDecimalsMismatch,
  #[msg("Invalid Metaplex Token Metadata program")]
  InvalidMetadataProgram,
  #[msg("Invalid metadata PDA")]
  InvalidMetadataPda,
  #[msg("Metaplex create metadata CPI failed")]
  MetadataCpiFailed,
}
