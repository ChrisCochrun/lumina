-- Add migration script here
CREATE TABLE IF NOT EXISTS 'songs' (
'id' INTEGER NOT NULL,
'title' TEXT NOT NULL,
'lyrics' TEXT,
'author' TEXT,
'ccli' TEXT,
'audio' TEXT,
'vorder' TEXT,
'background' TEXT,
'backgroundType' TEXT,
horizontalTextAlignment TEXT,
verticalTextAlignment TEXT,
font TEXT,
fontSize INTEGER,
PRIMARY KEY(id)
);

CREATE TABLE IF NOT EXISTS 'videos' (
'id' INTEGER NOT NULL,
'title' TEXT NOT NULL,
'filePath' TEXT NOT NULL,
startTime REAL,
endTime REAL,
loop BOOLEAN NOT NULL DEFAULT 0,
PRIMARY KEY(id)
);

CREATE TABLE IF NOT EXISTS 'images' (
'id' INTEGER NOT NULL,
'title' TEXT NOT NULL,
'filePath' TEXT NOT NULL,
PRIMARY KEY(id)
);

CREATE TABLE IF NOT EXISTS 'presentations' (
'id' INTEGER NOT NULL,
'title' TEXT NOT NULL,
'filePath' TEXT NOT NULL,
pageCount INTEGER DEFAULT 1,
html BOOLEAN NOT NULL DEFAULT 0,
PRIMARY KEY(id)
);
