use anchor_lang::error_code;

#[error_code]
pub enum OracleError {
  #[msg("Only oracle admin may call this instruction")]
  Unauthorized,
  #[msg("Price must be greater than zero")]
  InvalidPrice,
}
