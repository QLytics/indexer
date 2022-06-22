CREATE TABLE execution_outcomes (
  receipt_id TEXT PRIMARY KEY NOT NULL,
  block_hash TEXT NOT NULL,
  chunk_index INTEGER NOT NULL,
  timestamp DATETIME NOT NULL,
  gas_burnt TEXT(20) NOT NULL,
  tokens_burnt TEXT(45) NOT NULL,
  account_id TEXT NOT NULL,
  status TEXT CHECK( status IN ('UNKNOWN','FAILURE','SUCESS_VALUE', 'SUCCESS_RECEIPT_ID') ) NOT NULL,
  shard TEXT(20) NOT NULL
)
