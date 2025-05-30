DO
$$
    BEGIN
        IF EXISTS (SELECT 1 FROM pg_class WHERE relname = 'users') THEN
            EXECUTE 'DROP TABLE users CASCADE';
        END IF;
        IF EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_status') THEN
            EXECUTE 'DROP TYPE user_status';
        END IF;
    END
$$;

CREATE TYPE user_status AS ENUM
    (
        'active',
        'inactive',
        'suspended',
        'banned'
        );

CREATE TABLE users
(
    id         UUID PRIMARY KEY,
    name       VARCHAR(100)        NOT NULL,
    email      VARCHAR(255) UNIQUE NOT NULL,
    password   TEXT                NOT NULL,
    status     user_status         NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ         NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ         NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ                  DEFAULT NULL
);
