-- Add migration script here
ALTER TABLE images
RENAME COLUMN filePath TO file_path;

ALTER TABLE videos
RENAME COLUMN filePath TO file_path;

ALTER TABLE videos
RENAME COLUMN startTime TO start_time;

ALTER TABLE videos
RENAME COLUMN endTime TO end_time;

ALTER TABLE presentations
RENAME COLUMN filePath TO file_path;

ALTER TABLE presentations
RENAME COLUMN pageCount TO pageCount;

ALTER TABLE songs
RENAME COLUMN fontSize TO font_size;

ALTER TABLE songs
RENAME COLUMN vorder TO verse_order;

ALTER TABLE songs
RENAME COLUMN horizontalTextAlignment TO horizontal_text_alignment;

ALTER TABLE songs
RENAME COLUMN verticalTextAlignment TO vertical_text_alignment;

ALTER TABLE songs
RENAME COLUMN backgroundType TO background_type;
