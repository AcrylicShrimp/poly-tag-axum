-- Your SQL goes here

CREATE TABLE collections (
  id SERIAL PRIMARY KEY,
  uuid UUID NOT NULL DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL,
  description TEXT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX ON collections(uuid);
CREATE INDEX ON collections(name);
