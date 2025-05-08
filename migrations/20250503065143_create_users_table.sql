CREATE TYPE user_status AS ENUM
(
    'active',
    'inactive',
    'suspended',
    'banned'
)

CREATE TABLE users
(
    id         UUID PRIMARY KEY,
    email      VARCHAR(255) UNIQUE NOT NULL,
    password   TEXT                NOT NULL,
    status     user_status         NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ         NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ         NOT NULL DEFAULT NOW()
);
