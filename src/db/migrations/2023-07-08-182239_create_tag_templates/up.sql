-- Your SQL goes here
CREATE TYPE tag_value_type AS ENUM ('string', 'integer', 'boolean');

CREATE TABLE tag_templates (
  uuid UUID PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT NULL,
  value_type tag_value_type NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX ON tag_templates(created_at ASC);
CREATE INDEX ON tag_templates(name ASC);
