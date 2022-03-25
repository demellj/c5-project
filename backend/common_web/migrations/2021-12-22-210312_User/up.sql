-- Your SQL goes here
CREATE TABLE users (
    id SERIAL NOT NULL,
    email VARCHAR PRIMARY KEY NOT NULL,
    password_hash VARCHAR,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
)
