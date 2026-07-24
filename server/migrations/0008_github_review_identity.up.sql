ALTER TABLE github_installations
    ADD COLUMN review_identity_user_id TEXT,
    ADD COLUMN review_identity_login TEXT,
    ADD CONSTRAINT github_installations_review_identity_complete
        CHECK (
            (review_identity_user_id IS NULL AND review_identity_login IS NULL)
            OR (review_identity_user_id IS NOT NULL AND review_identity_login IS NOT NULL)
        );
