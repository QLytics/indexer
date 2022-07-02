CREATE TABLE chunks (
  chunk_hash TEXT PRIMARY KEY NOT NULL,
  block_hash TEXT NOT NULL,
  shard_id TEXT(20) NOT NULL,
  signature TEXT NOT NULL,
  gas_limit TEXT(20) NOT NULL,
  gas_used TEXT(20) NOT NULL,
  author_account_id TEXT NOT NULL
)
