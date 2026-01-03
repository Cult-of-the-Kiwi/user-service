CREATE TABLE IF NOT EXISTS friendships (
    from_user_id TEXT NOT NULL,
    to_user_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT friendships_pkey PRIMARY KEY (from_user_id, to_user_id),
    CONSTRAINT fk_from_user FOREIGN KEY (from_user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_to_user FOREIGN KEY (to_user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT no_self_friendship CHECK (from_user_id <> to_user_id)
);

CREATE INDEX IF NOT EXISTS friendships_ft_idx ON friendships (from_user_id, to_user_id);
CREATE INDEX IF NOT EXISTS friendships_to_idx ON friendships (to_user_id, from_user_id);
