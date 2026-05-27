-- Add migration script here
ALTER TABLE songs
ADD COLUMN created_at INTEGER;
ALTER TABLE songs
ADD COLUMN accessed_at INTEGER;

ALTER TABLE images
ADD COLUMN created_at INTEGER;
ALTER TABLE images
ADD COLUMN accessed_at INTEGER;

ALTER TABLE videos
ADD COLUMN created_at INTEGER;
ALTER TABLE videos
ADD COLUMN accessed_at INTEGER;

ALTER TABLE presentations
ADD COLUMN created_at INTEGER;
ALTER TABLE presentations
ADD COLUMN accessed_at INTEGER;
