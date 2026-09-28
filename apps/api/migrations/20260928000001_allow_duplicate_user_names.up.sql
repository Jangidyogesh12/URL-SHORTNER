-- Allow duplicate display names; uniqueness is enforced on email (and phone).
-- Original migration incorrectly declared `name varchar(50) unique not null`.
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_name_key;
