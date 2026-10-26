-- Add migration script here
ALTER TABLE presentations
ADD COLUMN starting_index INTEGER;

ALTER TABLE presentations
ADD COLUMN ending_index INTEGER;
