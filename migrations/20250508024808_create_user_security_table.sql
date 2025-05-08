CREATE TABLE user_security
(
    user_id                         UUID REFERENCES users(ID),
    mfa_enabled                     BOOLEAN             DEFAULT false,
    mfa_secret_key                  VARCHAR(255),
    last_password_change            TIMESTAMPZ,
    password_reset_token            VARCHAR(255),
    password_reset_expires_at       TIMESTAMPZ,
    failed_login_attempts           INTEGER             DEFAULT 0,
    last_login_failed               TIMESTAMPZ,
    account_locked_until            TIMESTAMPZ,
    created_at                      TIMESTAMPZ NOT NULL DEFAULT NOW(),
    updated_at                      TIMESTAMPZ NOT NULL DEFAULT NOW(),
)