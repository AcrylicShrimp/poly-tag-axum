-- Your SQL goes here
CREATE TABLE files (
  uuid UUID PRIMARY KEY,
  name TEXT NOT NULL,
  mime TEXT NULL,
  size BIGINT NULL,
  hash BIGINT NULL, -- sha256
  uploaded_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX ON files(uploaded_at ASC);
