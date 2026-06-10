ALTER TABLE user_identities
    ADD COLUMN provider_verified_at TIMESTAMPTZ;

UPDATE user_identities
SET provider_verified_at = updated_at
WHERE provider_verified_at IS NULL;

CREATE TABLE user_refresh_tokens (
    id UUID PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ
);

CREATE INDEX idx_user_refresh_tokens_user_id
    ON user_refresh_tokens(user_id);

CREATE INDEX idx_user_refresh_tokens_active_hash
    ON user_refresh_tokens(token_hash)
    WHERE revoked_at IS NULL;
