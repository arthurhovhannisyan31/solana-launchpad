#!/usr/bin/env node
// Run from repo root: node program/scripts/get-oracle-pda.js
// Or from program/: node scripts/get-oracle-pda.js
const anchor = require("@coral-xyz/anchor");
const {PublicKey} = anchor.web3;
const ORACLE_PROGRAM_ID = new PublicKey("24UJLhNSDEwFrziTkshg6Rt18K7H3RczKXR8fNpQ8xa3");
const [oraclePda] = PublicKey.findProgramAddressSync(
  [Buffer.from("oracle_state")],
  ORACLE_PROGRAM_ID
);
console.log("ORACLE_STATE_PUBKEY=" + oraclePda.toBase58());
