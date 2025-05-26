DO
$$
    BEGIN
        IF EXISTS (SELECT 1 FROM pg_class WHERE relname = 'user_sessions') THEN
            EXECUTE 'DROP TABLE user_sessions CASCADE';
        END IF;
    END
$$;

CREATE TABLE user_sessions
(
    id             UUID PRIMARY KEY,
    user_id        UUID REFERENCES users (ID),
    login_ip       INET         NOT NULL,
    login_device   VARCHAR(255) NOT NULL,
    login_location VARCHAR(255) NOT NULL,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at     TIMESTAMPTZ           DEFAULT NULL
)