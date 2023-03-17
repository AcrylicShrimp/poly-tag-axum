-- Your SQL goes here
CREATE TABLE stagings (
  uuid UUID PRIMARY KEY,
  staged_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX ON stagings(staged_at ASC);
