-- Add migration script here

CREATE TABLE articles(
  key BIGINT NOT NULL,
  data BYTEA
);