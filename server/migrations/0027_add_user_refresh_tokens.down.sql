DROP INDEX IF EXISTS idx_user_refresh_tokens_active_hash;
DROP INDEX IF EXISTS idx_user_refresh_tokens_user_id;
DROP TABLE IF EXISTS user_refresh_tokens;

ALTER TABLE user_identities
    DROP COLUMN IF EXISTS provider_verified_at;
