CREATE TABLE packages (
    id BINARY(16) PRIMARY KEY,
    name TINYTEXT NOT NULL,
    version TINYTEXT NOT NULL,
    license TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP NULL DEFAULT NULL
);