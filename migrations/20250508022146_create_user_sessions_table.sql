CREATE TABLE user_sessions
(
    id                  UUID PRIMARY KEY,
    user_id             UUID REFERENCES users(ID),
    login_ip            INET                NOT NULL,
    login_device        VARCHAR(255)        NOT NULL,
    login_location      VARCHAR(255)        NOT NULL,
    created_at          TIMESTAMPZ          NOT NULL    DEFAULT NOW(),
    updated_at          TIMESTAMPZ          NOT NULL    DEFAULT NOW()
)