-- Your SQL goes here
CREATE TABLE files (
  uuid UUID PRIMARY KEY,
  name TEXT NOT NULL,
  type TEXT NOT NULL,
  hash CHARACTER(80) NOT NULL,
  size BIGINT NOT NULL,
  uploaded_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX ON files(uploaded_at ASC);
