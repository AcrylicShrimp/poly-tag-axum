-- Your SQL goes here
CREATE TABLE files (
  uuid UUID PRIMARY KEY,
  file_name TEXT NOT NULL,
  file_type TEXT NOT NULL,
  file_hash CHARACTER(80) NOT NULL,
  file_size BIGINT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX ON files(created_at ASC);
