-- Add migration script here
CREATE TABLE IF NOT EXISTS friend_requests (
    from_user_id TEXT NOT NULL,
    to_user_id TEXT NOT NULL,
    state friend_request_state NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    responded_at TIMESTAMPTZ,

    CONSTRAINT friend_request_pkey PRIMARY KEY (from_user_id, to_user_id),
    CONSTRAINT fk_from_user FOREIGN KEY (from_user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_to_user FOREIGN KEY (to_user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT no_self_request CHECK (from_user_id <> to_user_id),
    CONSTRAINT unique_pending_request UNIQUE (from_user_id, to_user_id)
);
