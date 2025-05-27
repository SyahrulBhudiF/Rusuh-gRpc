# Justfile
run:
    cargo run

dev:
    cargo watch -x run

migrate:
    sqlx migrate run

create-db:
    sqlx database create

migrate-new name:
    sqlx migrate add {{name}}

drop-db:
    sqlx database drop

build:
    cargo build --release