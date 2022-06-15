table! {
    transactions (hash) {
        hash -> Text,
        block_hash -> Text,
        chunk_hash -> Text,
        chunk_index -> Int4,
        timestamp -> Numeric,
        signer_id -> Text,
        public_key -> Text,
        nonce -> Numeric,
        receiver_id -> Text,
        signature -> Text,
    }
}
