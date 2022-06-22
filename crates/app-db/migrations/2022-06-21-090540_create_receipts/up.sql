CREATE TABLE receipts (
  receipt_id TEXT PRIMARY KEY NOT NULL,
  block_hash TEXT NOT NULL,
  chunk_hash TEXT NOT NULL,
  chunk_index INTEGER NOT NULL,
  timestamp DATETIME NOT NULL,
  predecessor_id TEXT NOT NULL,
  receiver_id TEXT NOT NULL,
  receipt_kind TEXT CHECK( receipt_kind IN ('ACTION', 'DATA') ) NOT NULL
)
