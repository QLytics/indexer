table! {
    receipts (id) {
        id -> Text,
        block_hash -> Text,
        chunk_hash -> Text,
        chunk_index -> Integer,
        timestamp -> Timestamp,
        predecessor_id -> Text,
        receiver_id -> Text,
        receipt_kind -> Text,
    }
}

table! {
    transaction_actions (hash) {
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
        receipt_id -> Nullable<Text>,
        gas_burnt -> Nullable<Text>,
        tokens_burnt -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    receipts,
    transaction_actions,
    transactions,
);
