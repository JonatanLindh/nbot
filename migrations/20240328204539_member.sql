-- Add migration script here
CREATE TABLE IF NOT EXISTS member (
  id        BIGINT PRIMARY KEY,
  credits   BIGINT
);