ALTER TABLE versions
    DROP FOREIGN KEY FK_PackageVersion;

CREATE TABLE unique_names AS
SELECT MIN(id) AS id, name, created_at, updated_at, deleted_at
FROM packages
GROUP BY name;

TRUNCATE TABLE packages;

ALTER TABLE packages
    ADD CONSTRAINT UC_UniqueName UNIQUE (name);

INSERT INTO packages (id, name, created_at, updated_at, deleted_at)
SELECT id, name, created_at, updated_at, deleted_at
FROM unique_names;

DROP TABLE unique_names;

ALTER TABLE versions
    ADD CONSTRAINT FK_PackageVersion FOREIGN KEY (package_id) REFERENCES packages(id);