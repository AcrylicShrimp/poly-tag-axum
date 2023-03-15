-- Your SQL goes here
CREATE TABLE uploads (
    uuid UUID PRIMARY KEY,
    file_name VARCHAR(255),
    uploaded_size BIGINT NOT NULL DEFAULT 0,
    uploaded_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX ON uploads(uploaded_at ASC);
