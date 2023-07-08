-- Your SQL goes here
CREATE TABLE tags (
  uuid UUID PRIMARY KEY,
  template_uuid UUID NOT NULL REFERENCES tag_templates(uuid) ON UPDATE CASCADE ON DELETE CASCADE,
  file_uuid UUID NOT NULL REFERENCES files(uuid) ON UPDATE CASCADE ON DELETE CASCADE,
  value_string TEXT NULL,
  value_integer BIGINT NULL,
  value_boolean BOOLEAN NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX ON tags(created_at ASC);
