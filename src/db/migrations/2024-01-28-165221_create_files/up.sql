-- Your SQL goes here

CREATE TABLE files (
  id SERIAL PRIMARY KEY,
  uuid UUID NOT NULL DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL,
  mime TEXT NULL,
  size BIGINT NULL,
  hash BIGINT NULL, -- sha256
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX ON files(uuid);
