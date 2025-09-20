-- Add migration script here
CREATE TABLE IF NOT EXISTS blocks (
    from_user_id TEXT NOT NULL,
    to_user_id TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT blocks_pkey PRIMARY KEY (from_user_id, to_user_id),
    CONSTRAINT fk_from_user FOREIGN KEY (from_user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_to_user FOREIGN KEY (to_user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT no_self_block CHECK (from_user_id <> to_user_id)
);
