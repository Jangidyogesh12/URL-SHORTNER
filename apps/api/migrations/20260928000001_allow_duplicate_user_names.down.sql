-- Re-add unique constraint on users.name (may fail if duplicate names exist).
ALTER TABLE users ADD CONSTRAINT users_name_key UNIQUE (name);
