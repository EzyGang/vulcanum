ALTER TABLE github_installations
    DROP CONSTRAINT github_installations_review_identity_complete,
    DROP COLUMN review_identity_login,
    DROP COLUMN review_identity_user_id;
