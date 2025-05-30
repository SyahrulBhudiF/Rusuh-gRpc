DO
$$
    BEGIN
        IF EXISTS (SELECT 1 FROM pg_class WHERE relname = 'user_security') THEN
            EXECUTE 'DROP TABLE user_security CASCADE';
        END IF;
    END
$$;

CREATE TABLE user_security
(
    id                   UUID PRIMARY KEY,
    user_id              UUID REFERENCES users (ID),
    mfa_secret_key       VARCHAR(255),
    email_verified_at    TIMESTAMPTZ,
    last_password_change TIMESTAMPTZ,
    account_locked_until TIMESTAMPTZ,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at           TIMESTAMPTZ          DEFAULT NULL
)