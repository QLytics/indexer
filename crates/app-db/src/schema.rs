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
