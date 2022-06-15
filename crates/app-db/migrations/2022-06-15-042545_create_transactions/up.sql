CREATE TABLE transactions (
  hash TEXT PRIMARY KEY,
  block_hash TEXT NOT NULL,
  chunk_hash TEXT NOT NULL,
  chunk_index INTEGER NOT NULL,
  timestamp NUMERIC(20) NOT NULL,
  signer_id TEXT NOT NULL,
  public_key TEXT NOT NULL,
  nonce NUMERIC(20) NOT NULL,
  receiver_id TEXT NOT NULL,
  signature TEXT NOT NULL
)