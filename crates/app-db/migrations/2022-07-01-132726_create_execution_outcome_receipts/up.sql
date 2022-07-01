CREATE TABLE execution_outcome_receipts (
  receipt_id TEXT NOT NULL,
  index_in_execution_outcome INTEGER NOT NULL,
  produced_receipt_id TEXT NOT NULL,
  PRIMARY KEY (receipt_id, index_in_execution_outcome)
)
