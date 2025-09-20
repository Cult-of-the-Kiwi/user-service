-- Add migration script here
DO $$ BEGIN
    CREATE TYPE friend_request_state AS ENUM (
        'pending',
        'accepted',
        'rejected'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;
