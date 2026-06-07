DROP TABLE IF EXISTS github_installations;

CREATE TABLE github_installations (
    id BIGSERIAL PRIMARY KEY,
    account_login TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_github_installations_account_login ON github_installations(account_login);
