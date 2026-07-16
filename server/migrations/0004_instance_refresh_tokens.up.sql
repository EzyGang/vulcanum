CREATE TABLE instance_refresh_tokens (
    id UUID PRIMARY KEY,
    token_hash TEXT NOT NULL UNIQUE,
    password_fingerprint TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
