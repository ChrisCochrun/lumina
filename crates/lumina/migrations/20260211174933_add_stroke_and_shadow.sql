-- Add migration script here
ALTER TABLE songs
ADD COLUMN stroke_size INTEGER;

ALTER TABLE songs
ADD COLUMN stroke_color TEXT;

ALTER TABLE songs
ADD COLUMN shadow_size INTEGER;

ALTER TABLE songs
ADD COLUMN shadow_offset_x INTEGER;

ALTER TABLE songs
ADD COLUMN shadow_offset_y INTEGER;

ALTER TABLE songs
ADD COLUMN shadow_color TEXT;

