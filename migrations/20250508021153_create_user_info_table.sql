CREATE TYPE user_gender AS ENUM
(
    'male',
    'female',
    'other',
    'prefer_not_to_say'
)

CREATE TABLE user_info
(
    user_id     UUID         REFERENCES users(ID),
    first_name  VARCHAR(255) NOT NULL,
    last_name   VARCHAR(255) NOT NULL,
    gender      user_gender  NOT NULL,
    birth_date  DATE         NOT NULL,
    created_at  TIMESTAMPZ   NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPZ   NOT NULL DEFAULT NOW(),
)