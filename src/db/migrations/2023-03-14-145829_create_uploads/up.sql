-- Your SQL goes here
CREATE TABLE uploads (
    uuid UUID PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX ON uploads(created_at ASC);
