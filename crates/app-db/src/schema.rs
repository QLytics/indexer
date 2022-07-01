table! {
    data_receipts (data_id) {
        data_id -> Text,
        receipt_id -> Text,
        data -> Nullable<Binary>,
    }
}

table! {
    execution_outcome_receipts (receipt_id, index_in_execution_outcome) {
        receipt_id -> Text,
        index_in_execution_outcome -> Integer,
        produced_receipt_id -> Text,
    }
}

table! {
    execution_outcomes (receipt_id) {
        receipt_id -> Text,
        block_hash -> Text,
        chunk_index -> Integer,
        timestamp -> Timestamp,
        gas_burnt -> Text,
        tokens_burnt -> Text,
        account_id -> Text,
        status -> Text,
        shard -> Text,
    }
}

table! {
    receipts (receipt_id) {
        receipt_id -> Text,
        block_hash -> Text,
        chunk_hash -> Text,
        chunk_index -> Integer,
        timestamp -> Timestamp,
        predecessor_id -> Text,
        receiver_id -> Text,
        receipt_kind -> Text,
        transaction_hash -> Text,
    }
}

table! {
    transaction_actions (hash, transaction_index) {
        hash -> Text,
        transaction_index -> Integer,
        action_kind -> Text,
        args -> Text,
    }
}

table! {
    transactions (hash) {
        hash -> Text,
        block_hash -> Text,
        chunk_hash -> Text,
        chunk_index -> Integer,
        timestamp -> Timestamp,
        signer_id -> Text,
        public_key -> Text,
        nonce -> Text,
        receiver_id -> Text,
        signature -> Text,
        status -> Text,
        receipt_id -> Text,
        gas_burnt -> Text,
        tokens_burnt -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    data_receipts,
    execution_outcome_receipts,
    execution_outcomes,
    receipts,
    transaction_actions,
    transactions,
);
