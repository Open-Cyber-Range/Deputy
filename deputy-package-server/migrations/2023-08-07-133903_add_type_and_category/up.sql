ALTER TABLE packages
ADD COLUMN package_type TINYTEXT AFTER name;

UPDATE packages
SET package_type = 'Other'
WHERE packages.package_type IS NULL;

ALTER TABLE packages
MODIFY COLUMN package_type TINYTEXT NOT NULL;

CREATE TABLE categories (
    id BINARY(16) PRIMARY KEY,
    name TINYTEXT UNIQUE NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP NULL DEFAULT NULL
);

CREATE TABLE package_categories (
    package_id BINARY(16) NOT NULL,
    category_id BINARY(16) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP NULL DEFAULT NULL,
    PRIMARY KEY (package_id, category_id),
    CONSTRAINT FK_Category FOREIGN KEY (category_id) REFERENCES categories(id),
    CONSTRAINT FK_Package FOREIGN KEY (package_id) REFERENCES packages(id)
);