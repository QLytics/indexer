CREATE TABLE blocks (
  hash TEXT PRIMARY KEY NOT NULL,
  height TEXT(20) NOT NULL,
  prev_hash TEXT NOT NULL,
  timestamp DATETIME NOT NULL,
  total_supply TEXT(45) NOT NULL,
  gas_price TEXT(45) NOT NULL,
  author_account_id TEXT NOT NULL
)
