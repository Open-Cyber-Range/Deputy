ALTER TABLE versions
    ADD COLUMN is_yanked BOOLEAN NOT NULL DEFAULT FALSE AFTER license;