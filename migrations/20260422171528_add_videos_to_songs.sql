-- Add migration script here
ALTER TABLE songs
ADD COLUMN lyric_video TEXT;

ALTER TABLE songs
ADD COLUMN music_video TEXT;
