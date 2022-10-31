-- Your SQL goes here
CREATE TABLE packages (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    version VARCHAR NOT NULL,
    readme TEXT NOT NULL,
    licence VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);