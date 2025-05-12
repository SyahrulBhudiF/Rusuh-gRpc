CREATE TABLE user_security
(
    id                              UUID PRIMARY KEY,
    user_id                         UUID REFERENCES users(ID),
    mfa_enabled                     BOOLEAN             DEFAULT false,
    mfa_secret_key                  VARCHAR(255),
    email_verified_at               TIMESTAMPTZ,
    last_password_change            TIMESTAMPTZ,
    password_reset_token            VARCHAR(255),
    password_reset_expires_at       TIMESTAMPTZ,
    failed_login_attempts           INTEGER             DEFAULT 0,
    last_login_failed               TIMESTAMPTZ,
    account_locked_until            TIMESTAMPTZ,
    created_at                      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                      TIMESTAMPTZ NOT NULL DEFAULT NOW()
)