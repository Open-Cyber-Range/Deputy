ALTER TABLE packages
    ADD CONSTRAINT UC_UniqueName UNIQUE (name);