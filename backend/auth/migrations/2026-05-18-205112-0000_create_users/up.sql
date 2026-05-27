-- Your SQL goes here
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR NOT NULL,
    email VARCHAR NOT NULL UNIQUE,
    password_hash BYTEA NOT NULL,
    created_at TIMESTAMP DEFAULT current_timestamp NOT NULL
)