-- Your SQL goes here
CREATE TABLE tags (
  PRIMARY KEY(template_uuid, file_uuid),
  template_uuid UUID NOT NULL REFERENCES tag_templates(uuid) ON UPDATE CASCADE ON DELETE CASCADE,
  file_uuid UUID NOT NULL REFERENCES files(uuid) ON UPDATE CASCADE ON DELETE CASCADE,
  value_string TEXT NULL,
  value_integer BIGINT NULL,
  value_boolean BOOLEAN NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX ON tags(template_uuid ASC);
CREATE INDEX ON tags(file_uuid ASC);
CREATE INDEX ON tags(value_string ASC);
CREATE INDEX ON tags(value_integer ASC);
CREATE INDEX ON tags(value_boolean ASC);
CREATE INDEX ON tags(created_at ASC);
CREATE INDEX ON tags(template_uuid ASC, value_string ASC);
CREATE INDEX ON tags(template_uuid ASC, value_integer ASC);
CREATE INDEX ON tags(template_uuid ASC, value_boolean ASC);
