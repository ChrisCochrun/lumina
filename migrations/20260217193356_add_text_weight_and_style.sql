-- Add migration script here
ALTER TABLE songs
ADD COLUMN weight TEXT;

ALTER TABLE songs
ADD COLUMN style TEXT;
