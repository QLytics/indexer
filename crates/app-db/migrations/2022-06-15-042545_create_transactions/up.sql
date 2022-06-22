CREATE TABLE transactions (
  hash TEXT PRIMARY KEY NOT NULL,
  block_hash TEXT NOT NULL,
  chunk_hash TEXT NOT NULL,
  chunk_index INTEGER NOT NULL,
  timestamp DATETIME NOT NULL,
  signer_id TEXT NOT NULL,
  public_key TEXT NOT NULL,
  nonce TEXT(20) NOT NULL,
  receiver_id TEXT NOT NULL,
  signature TEXT NOT NULL,
  status TEXT CHECK( status IN ('UNKNOWN','FAILURE','SUCESS_VALUE', 'SUCCESS_RECEIPT_ID') ) NOT NULL,
  receipt_id TEXT NOT NULL,
  gas_burnt TEXT(20) NOT NULL,
  tokens_burnt TEXT(45) NOT NULL
)
