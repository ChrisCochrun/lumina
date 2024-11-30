use std::{collections::HashMap, path::PathBuf};

use cosmic::{executor, iced::Executor};
use crisp::types::{Keyword, Symbol, Value};
use miette::{miette, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use sqlx::{
    query, query_as, sqlite::SqliteRow, FromRow, Row,
    SqliteConnection,
};
use tracing::{debug, error};

use crate::core::slide;

use super::{
    model::Model,
    slide::{Background, TextAlignment},
};

#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize,
)]
pub struct Song {
    pub id: i32,
    pub title: String,
    pub lyrics: Option<String>,
    pub author: Option<String>,
    pub ccli: Option<String>,
    pub audio: Option<PathBuf>,
    pub verse_order: Option<Vec<String>>,
    pub background: Option<Background>,
    pub text_alignment: Option<TextAlignment>,
    pub font: Option<String>,
    pub font_size: Option<i32>,
}

const VERSE_KEYWORDS: [&str; 24] = [
    "Verse 1", "Verse 2", "Verse 3", "Verse 4", "Verse 5", "Verse 6",
    "Verse 7", "Verse 8", "Chorus 1", "Chorus 2", "Chorus 3",
    "Chorus 4", "Bridge 1", "Bridge 2", "Bridge 3", "Bridge 4",
    "Intro 1", "Intro 2", "Ending 1", "Ending 2", "Other 1",
    "Other 2", "Other 3", "Other 4",
];

impl FromRow<'_, SqliteRow> for Song {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get(12)?,
            title: row.try_get(5)?,
            lyrics: row.try_get(8)?,
            author: row.try_get(10)?,
            ccli: row.try_get(9)?,
            audio: Some(PathBuf::from({
                let string: String = row.try_get(11)?;
                string
            })),
            verse_order: Some({
                let str: &str = row.try_get(0)?;
                str.split(' ').map(|s| s.to_string()).collect()
            }),
            background: {
                let string: String = row.try_get(7)?;
                match Background::try_from(string) {
                    Ok(background) => Some(background),
                    Err(_) => None,
                }
            },
            text_alignment: Some({
                let horizontal_alignment: String = row.try_get(3)?;
                let vertical_alignment: String = row.try_get(4)?;
                match (
                    horizontal_alignment.to_lowercase().as_str(),
                    vertical_alignment.to_lowercase().as_str(),
                ) {
                    ("left", "top") => TextAlignment::TopLeft,
                    ("left", "center") => TextAlignment::MiddleLeft,
                    ("left", "bottom") => TextAlignment::BottomLeft,
                    ("center", "top") => TextAlignment::TopCenter,
                    ("center", "center") => {
                        TextAlignment::MiddleCenter
                    }
                    ("center", "bottom") => {
                        TextAlignment::BottomCenter
                    }
                    ("right", "top") => TextAlignment::TopRight,
                    ("right", "center") => TextAlignment::MiddleRight,
                    ("right", "bottom") => TextAlignment::BottomRight,
                    _ => TextAlignment::MiddleCenter,
                }
            }),
            font: row.try_get(6)?,
            font_size: row.try_get(1)?,
        })
    }
}

impl From<Value> for Song {
    fn from(value: Value) -> Self {
        match value {
            Value::List(list) => lisp_to_song(list),
            _ => Self::default(),
        }
    }
}

fn lisp_to_song(list: Vec<Value>) -> Song {
    const DEFAULT_SONG_ID: i32 = 0;
    const DEFAULT_SONG_LOCATION: usize = 0;

    // I probably shouldn't rely upon default locations for these
    // It's very unlikely that users of this method of song creation
    // would use them in this order, so instead, since these items are all
    // options, let's just treat them as if they don't exist if there isn't
    // a keyword found in the list.
    const DEFAULT_BACKGROUND_LOCATION: usize = 1;
    let background_position = if let Some(background) =
        list.iter().position(|v| {
            v == &Value::Keyword(Keyword::from("background"))
        }) {
        background + 1
    } else {
        DEFAULT_BACKGROUND_LOCATION
    };

    let background =
        if let Some(background) = list.get(background_position) {
            Some(slide::lisp_to_background(background))
        } else {
            None
        };

    const DEFAULT_AUTHOR_LOCATION: usize = 1;
    let author_position = if let Some(author) = list
        .iter()
        .position(|v| v == &Value::Keyword(Keyword::from("author")))
    {
        author + 1
    } else {
        DEFAULT_AUTHOR_LOCATION
    };

    let author = if let Some(author) = list.get(author_position) {
        Some(String::from(author))
    } else {
        None
    };

    const DEFAULT_CCLI_LOCATION: usize = 1;
    let ccli_position = if let Some(ccli) = list
        .iter()
        .position(|v| v == &Value::Keyword(Keyword::from("ccli")))
    {
        ccli + 1
    } else {
        DEFAULT_CCLI_LOCATION
    };

    let ccli = if let Some(ccli) = list.get(ccli_position) {
        Some(i32::from(ccli))
    } else {
        None
    };

    const DEFAULT_FONT_LOCATION: usize = 1;
    let font_position = if let Some(font) = list
        .iter()
        .position(|v| v == &Value::Keyword(Keyword::from("font")))
    {
        font + 1
    } else {
        DEFAULT_FONT_LOCATION
    };

    let font = if let Some(font) = list.get(font_position) {
        Some(String::from(font))
    } else {
        None
    };

    const DEFAULT_FONT_SIZE_LOCATION: usize = 1;
    let font_size_position = if let Some(font_size) =
        list.iter().position(|v| {
            v == &Value::Keyword(Keyword::from("font-size"))
        }) {
        font_size + 1
    } else {
        DEFAULT_FONT_SIZE_LOCATION
    };

    let font_size =
        if let Some(font_size) = list.get(font_size_position) {
            Some(i32::from(font_size))
        } else {
            None
        };

    const DEFAULT_TITLE_LOCATION: usize = 1;
    let title_position = if let Some(title) = list
        .iter()
        .position(|v| v == &Value::Keyword(Keyword::from("title")))
    {
        title + 1
    } else {
        DEFAULT_TITLE_LOCATION
    };

    let title = if let Some(title) = list.get(title_position) {
        String::from(title)
    } else {
        String::from("song")
    };

    const DEFAULT_VERSE_ORDER_LOCATION: usize = 1;
    let verse_order_position = if let Some(verse_order) =
        list.iter().position(|v| {
            v == &Value::Keyword(Keyword::from("verse-order"))
        }) {
        verse_order + 1
    } else {
        DEFAULT_VERSE_ORDER_LOCATION
    };

    let verse_order =
        if let Some(verse_order) = list.get(verse_order_position) {
            match verse_order {
                Value::List(vals) => Some(
                    vals.into_iter()
                        .map(|v| String::from(v))
                        .collect::<Vec<String>>(),
                ),
                _ => None,
            }
        } else {
            None
        };

    let first_text_postiion = if let Some(pos) =
        list.iter().position(|v| match v {
            Value::List(inner) => match &inner[0] {
                Value::Symbol(Symbol(text)) => {
                    text.contains("v1")
                        || text.contains("text")
                        || text.contains("c1")
                }
                _ => false,
            },
            _ => false,
        }) {
        pos
    } else {
        1
    };

    let lyric_elements = &list[first_text_postiion..];

    let mut lyric_list = if let Some(ref verse_order) = verse_order {
        Vec::with_capacity(verse_order.capacity())
    } else {
        vec![]
    };

    for element in lyric_elements {
        let Value::List(lyric) = element else {
            continue;
        };
        let Value::Symbol(Symbol(verse)) = &lyric[0] else {
            continue;
        };

        let lyric = String::from(&lyric[1]);
        let Some(ref verse_order) = verse_order else {
            lyric_list.push(lyric);
            continue;
        };

        let Some(verse_pos) =
            verse_order.iter().position(|v| v == verse)
        else {
            error!("Should be a verse here");
            continue;
        };

        lyric_list.insert(verse_pos, lyric);
    }

    todo!()
}

pub async fn get_song_from_db(
    index: i32,
    db: &mut SqliteConnection,
) -> Result<Song> {
    let row = query(r#"SELECT verse_order as "verse_order!", font_size as "font_size!: i32", background_type as "background_type!", horizontal_text_alignment as "horizontal_text_alignment!", vertical_text_alignment as "vertical_text_alignment!", title as "title!", font as "font!", background as "background!", lyrics as "lyrics!", ccli as "ccli!", author as "author!", audio as "audio!", id as "id: i32" from songs where id = $1"#).bind(index).fetch_one(db).await.into_diagnostic()?;
    Ok(Song::from_row(&row).into_diagnostic()?)
}

impl Model<Song> {
    pub async fn load_from_db(&mut self) {
        // static DATABASE_URL: &str = "sqlite:///home/chris/.local/share/lumina/library-db.sqlite3";
        let result = query(r#"SELECT verse_order as "verse_order!", font_size as "font_size!: i32", background_type as "background_type!", horizontal_text_alignment as "horizontal_text_alignment!", vertical_text_alignment as "vertical_text_alignment!", title as "title!", font as "font!", background as "background!", lyrics as "lyrics!", ccli as "ccli!", author as "author!", audio as "audio!", id as "id: i32"  from songs"#).fetch_all(&mut self.db).await;
        match result {
            Ok(s) => {
                for song in s.into_iter() {
                    match Song::from_row(&song) {
                        Ok(song) => {
                            let _ = self.add_item(song);
                        }
                        Err(e) => {
                            error!("Could not convert song: {e}")
                        }
                    };
                }
            }
            Err(e) => {
                error!("There was an error in converting songs: {e}");
            }
        }
    }
}

impl Song {
    pub fn get_lyrics(&self) -> Result<Vec<String>> {
        let mut lyric_list = Vec::new();
        if self.lyrics.is_none() {
            return Err(miette!("There is no lyrics here"));
        } else if self.verse_order.is_none() {
            return Err(miette!("There is no verse_order here"));
        } else if self
            .verse_order
            .clone()
            .is_some_and(|v| v.is_empty())
        {
            return Err(miette!("There is no verse_order here"));
        }
        if let Some(raw_lyrics) = self.lyrics.clone() {
            let raw_lyrics = raw_lyrics.as_str();
            let verse_order = self.verse_order.clone();

            let mut lyric_map = HashMap::new();
            let mut verse_title = String::from("");
            let mut lyric = String::from("");
            for (i, line) in raw_lyrics.split('\n').enumerate() {
                if VERSE_KEYWORDS.contains(&line) {
                    if i != 0 {
                        lyric_map.insert(verse_title, lyric);
                        lyric = String::from("");
                        verse_title = line.to_string();
                    } else {
                        verse_title = line.to_string();
                    }
                } else {
                    lyric.push_str(line);
                    lyric.push('\n');
                }
            }
            lyric_map.insert(verse_title, lyric);

            for verse in verse_order.unwrap_or_default() {
                let mut verse_name = "";
                debug!(verse = verse);
                for word in VERSE_KEYWORDS {
                    let end_verse =
                        verse.get(1..2).unwrap_or_default();
                    let beg_verse =
                        verse.get(0..1).unwrap_or_default();
                    if word.starts_with(beg_verse)
                        && word.ends_with(end_verse)
                    {
                        verse_name = word;
                        continue;
                    }
                }
                if let Some(lyric) = lyric_map.get(verse_name) {
                    if lyric.contains("\n\n") {
                        let split_lyrics: Vec<&str> =
                            lyric.split("\n\n").collect();
                        for lyric in split_lyrics {
                            if lyric.is_empty() {
                                continue;
                            }
                            lyric_list.push(lyric.to_string());
                        }
                        continue;
                    }
                    lyric_list.push(lyric.to_string());
                } else {
                    error!("NOT WORKING!");
                };
            }
            for lyric in lyric_list.iter() {
                debug!(lyric = ?lyric)
            }
            Ok(lyric_list)
        } else {
            Err(miette!("There are no lyrics"))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    pub fn test_song_lyrics() {
        let mut song = Song::default();
        song.lyrics = Some(
            "Verse 1
When You found me,
I was so blind
My sin was before me,
I was swallowed by pride

Chorus 1
But out of the darkness,
You brought me to Your light
You showed me new mercy
And opened up my eyes

Chorus 2
From the day
You saved my soul
'Til the very moment
When I come home

I'll sing, I'll dance,
My heart will overflow
From the day
You saved my soul

Verse 2
Where brilliant light
Is all around
And endless joy
Is the only sound

Chorus 3
Oh, rest my heart
Forever now
Oh, in Your arms
I'll always be found

Bridge 1
My love is Yours
My heart is Yours
My life is Yours
Forever

My love is Yours
My heart is Yours
My life is Yours
Forever

Other 1
From the Day
I Am They

Other 2


Ending 1
Oh Oh Oh
From the day
You saved my soul"
                .to_string(),
        );
        song.verse_order =
            "O1 V1 C1 C2 O2 V2 C3 C2 O2 B1 C2 C2 E1 O2"
                .to_string()
                .split(' ')
                .map(|s| Some(s.to_string()))
                .collect();
        let lyrics = song.get_lyrics();
        match lyrics {
            Ok(lyrics) => {
                assert_eq!(vec!["From the Day\nI Am They", "When You found me,\nI was so blind\nMy sin was before me,\nI was swallowed by pride", "But out of the darkness,\nYou brought me to Your light\nYou showed me new mercy\nAnd opened up my eyes", "From the day\nYou saved my soul\n'Til the very moment\nWhen I come home", "I'll sing, I'll dance,\nMy heart will overflow\nFrom the day\nYou saved my soul", "Where brilliant light\nIs all around\nAnd endless joy\nIs the only sound", "Oh, rest my heart\nForever now\nOh, in Your arms\nI'll always be found", "From the day\nYou saved my soul\n'Til the very moment\nWhen I come home", "I'll sing, I'll dance,\nMy heart will overflow\nFrom the day\nYou saved my soul", "My love is Yours\nMy heart is Yours\nMy life is Yours\nForever", "My love is Yours\nMy heart is Yours\nMy life is Yours\nForever", "From the day\nYou saved my soul\n'Til the very moment\nWhen I come home", "I'll sing, I'll dance,\nMy heart will overflow\nFrom the day\nYou saved my soul", "From the day\nYou saved my soul\n'Til the very moment\nWhen I come home", "I'll sing, I'll dance,\nMy heart will overflow\nFrom the day\nYou saved my soul", "Oh Oh Oh\nFrom the day\nYou saved my soul\n"], lyrics);
            }
            Err(e) => {
                assert!(false, "{:?}", e)
            }
        }
    }

    async fn model() -> Model<Song> {
        let song_model: Model<Song> = Model {
            items: vec![],
            db: crate::core::model::get_db().await,
        };
        song_model
    }

    #[tokio::test]
    async fn test_db_and_model() {
        let mut song_model = model().await;
        song_model.load_from_db().await;
        if let Some(song) = song_model.find(|s| s.id == 7) {
            let test_song = test_song();
            assert_eq!(&test_song, song);
        } else {
            dbg!(song_model);
            assert!(false);
        }
    }

    #[tokio::test]
    async fn test_song_from_db() {
        let song = test_song();
        let mut db = model().await.db;
        let result = get_song_from_db(7, &mut db).await;
        match result {
            Ok(db_song) => assert_eq!(song, db_song),
            Err(e) => assert!(false, "{e}"),
        }
    }

    #[tokio::test]
    async fn test_update() {
        let song = test_song();
        let cloned_song = song.clone();
        let mut song_model: Model<Song> = model().await;
        song_model.load_from_db().await;

        match song_model.update_item(song, 2) {
            Ok(()) => assert_eq!(
                &cloned_song,
                song_model.find(|s| s.id == 7).unwrap()
            ),
            Err(e) => assert!(false, "{e}"),
        }
    }

    fn test_song() -> Song {
        Song {
            id: 7,
            title: "Death Was Arrested".to_string(),
            lyrics: Some("Intro 1\nDeath Was Arrested\nNorth Point Worship\n\nVerse 1\nAlone in my sorrow\nAnd dead in my sin\n\nLost without hope\nWith no place to begin\n\nYour love made a way\nTo let mercy come in\n\nWhen death was arrested\nAnd my life began\n\nVerse 2\nAsh was redeemed\nOnly beauty remains\n\nMy orphan heart\nWas given a name\n\nMy mourning grew quiet,\nMy feet rose to dance\n\nWhen death was arrested\nAnd my life began\n\nChorus 1\nOh, Your grace so free,\nWashes over me\n\nYou have made me new,\nNow life begins with You\n\nIt's Your endless love,\nPouring down on us\n\nYou have made us new,\nNow life begins with You\n\nVerse 3\nReleased from my chains,\nI'm a prisoner no more\n\nMy shame was a ransom\nHe faithfully bore\n\nHe cancelled my debt and\nHe called me His friend\n\nWhen death was arrested\nAnd my life began\n\nVerse 4\nOur Savior displayed\nOn a criminal's cross\n\nDarkness rejoiced as though\nHeaven had lost\n\nBut then Jesus arose\nWith our freedom in hand\n\nThat's when death was arrested\nAnd my life began\n\nThat's when death was arrested\nAnd my life began\n\nBridge 1\nOh, we're free, free,\nForever we're free\n\nCome join the song\nOf all the redeemed\n\nYes, we're free, free,\nForever amen\n\nWhen death was arrested\nAnd my life began\n\nOh, we're free, free,\nForever we're free\n\nCome join the song\nOf all the redeemed\n\nYes, we're free, free,\nForever amen\n\nWhen death was arrested\nAnd my life began\n\nEnding 1\nWhen death was arrested\nAnd my life began\n\nThat's when death was arrested\nAnd my life began".to_string()),
            author: Some(
                "North Point Worship".to_string(),
            ),
            ccli: None,
            audio: Some("file:///home/chris/music/North Point InsideOut/Nothing Ordinary, Pt. 1 (Live)/05 Death Was Arrested (feat. Seth Condrey).mp3".into()),
            verse_order: Some(vec![
                "I1".to_string(),
                "V1".to_string(),
                "V2".to_string(),
                "C1".to_string(),
                "V3".to_string(),
                "C1".to_string(),
                "V4".to_string(),
                "C1".to_string(),
                "B1".to_string(),
                "B1".to_string(),
                "E1".to_string(),
                "E2".to_string(),
            ]),
            background: Some(Background::try_from("file:///home/chris/nc/tfc/openlp/CMG - Bright Mountains 01.jpg").unwrap()),
            text_alignment: Some(TextAlignment::MiddleCenter),
            font: Some("Quicksand Bold".to_string()),
            font_size: Some(60)
        }
    }
}
