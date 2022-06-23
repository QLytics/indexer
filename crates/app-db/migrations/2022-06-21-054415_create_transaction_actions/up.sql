CREATE TABLE transaction_actions (
  hash TEXT NOT NULL,
  transaction_index INTEGER NOT NULL,
  action_kind TEXT CHECK( action_kind IN ('CREATE_ACCOUNT', 'DEPLOY_CONTRACT', 'FUNCTION_CALL', 'TRANSFER', 'STAKE', 'ADD_KEY', 'DELETE_KEY', 'DELETE_ACCOUNT') ) NOT NULL,
  args TEXT NOT NULL,
  PRIMARY KEY (hash, transaction_index)
)
