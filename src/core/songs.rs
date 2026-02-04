use std::{
    borrow::Cow, collections::HashMap, option::Option, path::PathBuf,
};

use cosmic::{
    cosmic_theme::palette::rgb::Rgba,
    iced::clipboard::mime::AsMimeTypes,
};
use crisp::types::{Keyword, Symbol, Value};
use itertools::Itertools;
use miette::{IntoDiagnostic, Result, miette};
use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow, Row, Sqlite, SqliteConnection, SqlitePool,
    pool::PoolConnection, query, sqlite::SqliteRow,
};
use tracing::{debug, error};

use crate::{Slide, SlideBuilder, core::slide};

use super::{
    content::Content,
    kinds::ServiceItemKind,
    model::{LibraryKind, Model},
    service_items::ServiceTrait,
    slide::{Background, TextAlignment},
};

#[derive(
    Clone, Debug, Default, PartialEq, Serialize, Deserialize,
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
    pub stroke_size: Option<i32>,
    pub stroke_color: Option<Rgba>,
    pub shadow_size: Option<i32>,
    pub shadow_offset: Option<(i32, i32)>,
    pub shadow_color: Option<Rgba>,
    pub verses: Option<Vec<VerseName>>,
    pub verse_map: Option<HashMap<VerseName, String>>,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Hash,
    PartialOrd,
    Ord,
)]
pub enum VerseName {
    Verse { number: usize },
    PreChorus { number: usize },
    Chorus { number: usize },
    PostChorus { number: usize },
    Bridge { number: usize },
    Intro { number: usize },
    Outro { number: usize },
    Instrumental { number: usize },
    Other { number: usize },
    Blank,
}

impl VerseName {
    pub fn from_string(name: String) -> Self {
        match name.as_str() {
            "Verse" => Self::Verse { number: 1 },
            "Pre-Chorus" => Self::PreChorus { number: 1 },
            "Chorus" => Self::Chorus { number: 1 },
            "Post-Chorus" => Self::PostChorus { number: 1 },
            "Bridge" => Self::Bridge { number: 1 },
            "Intro" => Self::Intro { number: 1 },
            "Outro" => Self::Outro { number: 1 },
            "Instrumental" => Self::Instrumental { number: 1 },
            "Other" => Self::Other { number: 1 },
            "Blank" => Self::Blank,
            _ => Self::Blank,
        }
    }

    pub fn all_names() -> Vec<String> {
        vec![
            "Verse".into(),
            "Pre-Chorus".into(),
            "Chorus".into(),
            "Post-Chorus".into(),
            "Bridge".into(),
            "Intro".into(),
            "Outro".into(),
            "Instrumental".into(),
            "Other".into(),
            "Blank".into(),
        ]
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Verse { number } => {
                Self::Verse { number: number + 1 }
            }
            Self::PreChorus { number } => {
                Self::PreChorus { number: number + 1 }
            }
            Self::Chorus { number } => {
                Self::Chorus { number: number + 1 }
            }
            Self::PostChorus { number } => {
                Self::PostChorus { number: number + 1 }
            }
            Self::Bridge { number } => {
                Self::Bridge { number: number + 1 }
            }
            Self::Intro { number } => {
                Self::Intro { number: number + 1 }
            }
            Self::Outro { number } => {
                Self::Outro { number: number + 1 }
            }
            Self::Instrumental { number } => {
                Self::Instrumental { number: number + 1 }
            }
            Self::Other { number } => {
                Self::Other { number: number + 1 }
            }
            Self::Blank => Self::Blank,
        }
    }

    #[must_use]
    pub fn get_name(&self) -> String {
        match self {
            Self::Verse { number, .. } => {
                let mut string = "Verse ".to_string();
                string.push_str(&number.to_string());
                string
            }
            Self::PreChorus { number, .. } => {
                let mut string = "Pre-Chorus ".to_string();
                string.push_str(&number.to_string());
                string
            }
            Self::Chorus { number, .. } => {
                let mut string = "Chorus ".to_string();
                string.push_str(&number.to_string());
                string
            }
            Self::PostChorus { number, .. } => {
                let mut string = "Post-Chorus ".to_string();
                string.push_str(&number.to_string());
                string
            }
            Self::Bridge { number, .. } => {
                let mut string = "Bridge ".to_string();
                string.push_str(&number.to_string());
                string
            }
            Self::Intro { number, .. } => {
                let mut string = "Intro ".to_string();
                string.push_str(&number.to_string());
                string
            }
            Self::Outro { number, .. } => {
                let mut string = "Outro ".to_string();
                string.push_str(&number.to_string());
                string
            }
            Self::Instrumental { number, .. } => {
                let mut string = "Instrumental ".to_string();
                string.push_str(&number.to_string());
                string
            }
            Self::Other { number, .. } => {
                let mut string = "Other ".to_string();
                string.push_str(&number.to_string());
                string
            }
            Self::Blank => "Blank".to_string(),
        }
    }
}

impl TryFrom<(Vec<u8>, String)> for VerseName {
    type Error = miette::Error;

    fn try_from(
        value: (Vec<u8>, String),
    ) -> std::result::Result<Self, Self::Error> {
        let (data, mime) = value;
        debug!(?mime);
        ron::de::from_bytes(&data).into_diagnostic()
    }
}

impl AsMimeTypes for VerseName {
    fn available(&self) -> std::borrow::Cow<'static, [String]> {
        Cow::from(vec!["application/verse".to_string()])
    }

    fn as_bytes(
        &self,
        _mime_type: &str,
    ) -> Option<std::borrow::Cow<'static, [u8]>> {
        let ron = ron::ser::to_string(self).ok()?;
        Some(Cow::from(ron.into_bytes()))
    }
}

impl Default for VerseName {
    fn default() -> Self {
        Self::Verse { number: 1 }
    }
}

impl From<&Song> for Value {
    fn from(_value: &Song) -> Self {
        Self::List(vec![Self::Symbol(Symbol("song".into()))])
    }
}

impl Content for Song {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn kind(&self) -> ServiceItemKind {
        ServiceItemKind::Song(self.clone())
    }

    fn to_service_item(&self) -> super::service_items::ServiceItem {
        self.into()
    }

    fn background(&self) -> Option<Background> {
        self.background.clone()
    }

    fn subtext(&self) -> String {
        self.author.clone().unwrap_or("Author missing".into())
    }
}

impl ServiceTrait for Song {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn id(&self) -> i32 {
        self.id
    }

    fn to_slides(&self) -> Result<Vec<Slide>> {
        // let lyrics = self.get_lyrics()?;
        let lyrics: Vec<String> = if let Some(verses) =
            self.verses.as_ref()
            && let Some(map) = self.verse_map.as_ref()
        {
            verses
                .iter()
                .filter_map(|verse| {
                    map.get(verse).map(|lyric| {
                        let lyric =
                            lyric.to_owned().trim().to_string();
                        let multi_lyric = lyric.split("\n\n");
                        let lyric: Vec<String> = multi_lyric
                            .map(|lyric| lyric.trim().to_string())
                            .collect();
                        lyric
                    })
                })
                .flatten()
                .collect()
        } else {
            vec![]
        };
        debug!(?lyrics);
        let slides: Vec<Slide> = lyrics
            .iter()
            .map(|l| {
                SlideBuilder::new()
                    .background(
                        self.background.clone().unwrap_or_default(),
                    )
                    .font(self.font.clone().unwrap_or_default())
                    .font_size(self.font_size.unwrap_or_default())
                    .text_alignment(
                        self.text_alignment.unwrap_or_default(),
                    )
                    .audio(self.audio.clone().unwrap_or_default())
                    .video_loop(true)
                    .video_start_time(0.0)
                    .video_end_time(0.0)
                    .text(l)
                    .build()
                    .unwrap_or_default()
            })
            .collect();

        Ok(slides)
    }

    fn box_clone(&self) -> Box<dyn ServiceTrait> {
        Box::new((*self).clone())
    }
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
        let lyrics: String = row.try_get(8)?;
        // let Some((mut verses, mut verse_map)) =
        //     lyrics_to_verse(lyrics.clone()).ok()
        // else {
        //     return Err(sqlx::Error::ColumnDecode {
        //         index: "8".into(),
        //         source: miette!(
        //             "Couldn't decode the song into verses"
        //         )
        //         .into(),
        //     });
        // };

        let Ok(verse_map) = ron::de::from_str::<
            Option<HashMap<VerseName, String>>,
        >(lyrics.as_ref()) else {
            return Err(sqlx::Error::ColumnDecode {
                index: "8".into(),
                source: miette!(
                    "Couldn't decode the song into verses"
                )
                .into(),
            });
        };
        let verse_order: &str = row.try_get(0)?;
        let Ok(verses) =
            ron::de::from_str::<Option<Vec<VerseName>>>(verse_order)
        else {
            return Err(sqlx::Error::ColumnDecode {
                index: "0".into(),
                source: miette!(
                    "Couldn't decode the song into verses"
                )
                .into(),
            });
        };

        let verse_order: Vec<String> = {
            verse_order
                .split(' ')
                .map(std::string::ToString::to_string)
                .collect()
        };

        Ok(Self {
            id: row.try_get(12)?,
            title: row.try_get(5)?,
            lyrics: Some(lyrics),
            author: row.try_get(10)?,
            ccli: row.try_get(9)?,
            audio: Some(PathBuf::from({
                let string: String = row.try_get(11)?;
                string
            })),
            verse_order: Some(verse_order),
            background: {
                let string: String = row.try_get(7)?;
                Background::try_from(string).ok()
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
            stroke_size: None,
            verses,
            verse_map,
            ..Default::default()
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

fn lyrics_to_verse(
    lyrics: String,
) -> Result<(Vec<VerseName>, HashMap<VerseName, String>)> {
    let mut verse_list = Vec::new();
    if lyrics.is_empty() {
        return Err(miette!("There is no lyrics here"));
    }

    let raw_lyrics = lyrics.as_str();

    let mut lyric_map = HashMap::new();
    let mut verse_title = String::new();
    let mut lyric = String::new();
    for (i, line) in raw_lyrics.split('\n').enumerate() {
        if VERSE_KEYWORDS.contains(&line) {
            if i != 0 {
                lyric_map.insert(verse_title, lyric);
                lyric = String::new();
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
    let mut verse_map = HashMap::new();
    for (verse_name, lyric) in lyric_map {
        let mut verse_elements = verse_name.split_whitespace();
        let verse_keyword = verse_elements.next();
        let Some(keyword) = verse_keyword else {
            return Err(miette!(
                "Can't parse a proper verse keyword from lyrics"
            ));
        };
        let verse_index = verse_elements.next();
        let Some(index) = verse_index else {
            return Err(miette!(
                "Can't parse a proper verse index from lyrics"
            ));
        };
        let index = index.parse::<usize>().into_diagnostic()?;
        let verse = match keyword {
            "Verse" => VerseName::Verse { number: index },
            "Pre-Chorus" => VerseName::PreChorus { number: index },
            "Chorus" => VerseName::Chorus { number: index },
            "Post-Chorus" => VerseName::PostChorus { number: index },
            "Bridge" => VerseName::Bridge { number: index },
            "Intro" => VerseName::Intro { number: index },
            "Outro" => VerseName::Outro { number: index },
            "Instrumental" => {
                VerseName::Instrumental { number: index }
            }
            "Other" => VerseName::Other { number: index },
            _ => VerseName::Other { number: 99 },
        };
        verse_list.push(verse);
        let lyric = lyric.trim().to_string();
        verse_map.insert(verse, lyric);
    }

    Ok((verse_list, verse_map))
}

pub fn lisp_to_song(list: Vec<Value>) -> Song {
    const DEFAULT_SONG_ID: i32 = 0;
    // const DEFAULT_SONG_LOCATION: usize = 0;

    let id = if let Some(key_pos) = list
        .iter()
        .position(|v| v == &Value::Keyword(Keyword::from("id")))
    {
        let pos = key_pos + 1;
        list.get(pos).map(i32::from).unwrap_or_default()
    } else {
        DEFAULT_SONG_ID
    };

    let background = if let Some(key_pos) =
        list.iter().position(|v| {
            v == &Value::Keyword(Keyword::from("background"))
        }) {
        let pos = key_pos + 1;
        list.get(pos).map(slide::lisp_to_background)
    } else {
        None
    };

    let author = if let Some(key_pos) = list
        .iter()
        .position(|v| v == &Value::Keyword(Keyword::from("author")))
    {
        let pos = key_pos + 1;
        list.get(pos).map(String::from)
    } else {
        None
    };

    let audio = if let Some(key_pos) = list
        .iter()
        .position(|v| v == &Value::Keyword(Keyword::from("audio")))
    {
        let pos = key_pos + 1;
        list.get(pos).map(|a| PathBuf::from(String::from(a)))
    } else {
        None
    };

    let ccli = if let Some(key_pos) = list
        .iter()
        .position(|v| v == &Value::Keyword(Keyword::from("ccli")))
    {
        let pos = key_pos + 1;
        list.get(pos).map(|c| i32::from(c).to_string())
    } else {
        None
    };

    let font = if let Some(key_pos) = list
        .iter()
        .position(|v| v == &Value::Keyword(Keyword::from("font")))
    {
        let pos = key_pos + 1;
        list.get(pos).map(String::from)
    } else {
        None
    };

    let font_size = if let Some(key_pos) = list.iter().position(|v| {
        v == &Value::Keyword(Keyword::from("font-size"))
    }) {
        let pos = key_pos + 1;
        list.get(pos).map(i32::from)
    } else {
        None
    };

    let title = if let Some(key_pos) = list
        .iter()
        .position(|v| v == &Value::Keyword(Keyword::from("title")))
    {
        let pos = key_pos + 1;
        list.get(pos).map_or(String::from("song"), String::from)
    } else {
        String::from("song")
    };

    let text_alignment = if let Some(key_pos) =
        list.iter().position(|v| {
            v == &Value::Keyword(Keyword::from("text-alignment"))
        }) {
        let pos = key_pos + 1;
        list.get(pos).map(TextAlignment::from)
    } else {
        None
    };

    let verse_order = if let Some(key_pos) =
        list.iter().position(|v| {
            v == &Value::Keyword(Keyword::from("verse-order"))
        }) {
        let pos = key_pos + 1;
        list.get(pos).map(|v| match v {
            Value::List(vals) => vals
                .iter()
                .map(|v| String::from(v).to_uppercase())
                .collect::<Vec<String>>(),
            _ => vec![],
        })
    } else {
        None
    };

    let first_text_postiion = list
        .iter()
        .position(|v| match v {
            Value::List(inner) => {
                (match &inner[0] {
                    Value::Symbol(Symbol(text)) => {
                        text.contains("v1")
                            || text.contains("text")
                            || text.contains("c1")
                            || text.contains("i1")
                    }
                    _ => false,
                } && match &inner[1] {
                    Value::String(_) => true,
                    _ => false,
                })
            }
            _ => false,
        })
        .unwrap_or(1);

    let lyric_elements = &list[first_text_postiion..];

    let mut lyrics = vec![];

    for element in lyric_elements {
        let Value::List(lyric) = element else {
            continue;
        };
        let Value::Symbol(Symbol(lyric_verse)) = &lyric[0] else {
            continue;
        };

        let lyric = String::from(&lyric[1]);

        let verse_title = match lyric_verse.as_str() {
            "i1" => r"\n\nIntro 1\n",
            "i2" => r"\n\nIntro 1\n",
            "v1" => r"\n\nVerse 1\n",
            "v2" => r"\n\nVerse 2\n",
            "v3" => r"\n\nVerse 3\n",
            "v4" => r"\n\nVerse 4\n",
            "v5" => r"\n\nVerse 5\n",
            "c1" => r"\n\nChorus 1\n",
            "c2" => r"\n\nChorus 2\n",
            "c3" => r"\n\nChorus 3\n",
            "c4" => r"\n\nChorus 4\n",
            "b1" => r"\n\nBridge 1\n",
            "b2" => r"\n\nBridge 2\n",
            "e1" => r"\n\nEnding 1\n",
            "e2" => r"\n\nEnding 2\n",
            "o1" => r"\n\nOther 1\n",
            "o2" => r"\n\nOther 2\n",
            _ => "",
        };

        let lyric = format!("{verse_title}{lyric}");
        let lyric = lyric.replace(
            "\\n", r"
",
        );
        lyrics.push(lyric);
    }

    let lyrics: String =
        lyrics.iter().flat_map(|s| s.chars()).collect();
    let lyrics = lyrics.trim_start().to_string();

    Song {
        id,
        title,
        lyrics: Some(lyrics),
        author,
        audio,
        ccli,
        verse_order,
        background,
        font,
        font_size,
        text_alignment,
        ..Default::default()
    }
}

pub async fn get_song_from_db(
    index: i32,
    db: &mut SqliteConnection,
) -> Result<Song> {
    let row = query(r#"SELECT verse_order as "verse_order!", font_size as "font_size!: i32", background_type as "background_type!", horizontal_text_alignment as "horizontal_text_alignment!", vertical_text_alignment as "vertical_text_alignment!", title as "title!", font as "font!", background as "background!", lyrics as "lyrics!", ccli as "ccli!", author as "author!", audio as "audio!", id as "id: i32" from songs where id = $1"#).bind(index).fetch_one(db).await.into_diagnostic()?;
    Song::from_row(&row).into_diagnostic()
}

impl Model<Song> {
    pub async fn new_song_model(db: &mut SqlitePool) -> Self {
        let mut model = Self {
            items: vec![],
            kind: LibraryKind::Song,
        };

        model.load_from_db(db).await;
        model
    }

    pub async fn load_from_db(&mut self, db: &mut SqlitePool) {
        // static DATABASE_URL: &str = "sqlite:///home/chris/.local/share/lumina/library-db.sqlite3";
        let db1 = db.acquire().await.unwrap();
        let result = query(r#"SELECT verse_order as "verse_order!", font_size as "font_size!: i32", background_type as "background_type!", horizontal_text_alignment as "horizontal_text_alignment!", vertical_text_alignment as "vertical_text_alignment!", title as "title!", font as "font!", background as "background!", lyrics as "lyrics!", ccli as "ccli!", author as "author!", audio as "audio!", id as "id: i32"  from songs"#).fetch_all(&mut db1.detach()).await;
        match result {
            Ok(s) => {
                for song in s {
                    let db2 = db.acquire().await.unwrap();
                    match Song::from_row(&song) {
                        Ok(song) => {
                            match update_song_in_db(song.clone(), db2)
                                .await
                            {
                                Ok(_) => {
                                    let _ = self.add_item(song);
                                }
                                Err(e) => error!(?e),
                            }
                        }
                        Err(e) => {
                            error!(
                                song_empty = song.is_empty(),
                                "Could not convert song: {e}: the song is likely broken from an old format if it isn't empty"
                            );
                        }
                    }
                }
            }
            Err(e) => {
                error!("There was an error in converting songs: {e}");
            }
        }
    }
}

pub async fn remove_from_db(
    db: PoolConnection<Sqlite>,
    id: i32,
) -> Result<()> {
    query!("DELETE FROM songs WHERE id = $1", id)
        .execute(&mut db.detach())
        .await
        .into_diagnostic()
        .map(|_| ())
}

pub async fn add_song_to_db(
    db: PoolConnection<Sqlite>,
) -> Result<Song> {
    let mut db = db.detach();
    let mut song = Song::default();

    let verse_order = {
        if let Some(vo) = song.verse_order.clone() {
            vo.into_iter()
                .map(|mut s| {
                    s.push(' ');
                    s
                })
                .collect::<String>()
        } else {
            String::new()
        }
    };

    let audio = song
        .audio
        .clone()
        .map(|a| a.to_str().unwrap_or_default().to_string());

    let background = song
        .background
        .clone()
        .map(|b| b.path.to_str().unwrap_or_default().to_string());

    let res = query!(
        r#"INSERT INTO songs (title, lyrics, author, ccli, verse_order, audio, font, font_size, background) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        song.title,
        song.lyrics,
        song.author,
        song.ccli,
        verse_order,
        audio,
        song.font,
        song.font_size,
        background
    )
    .execute(&mut db)
    .await
    .into_diagnostic()?;
    song.id = res.last_insert_rowid() as i32;
    Ok(song)
}

pub async fn update_song_in_db(
    item: Song,
    db: PoolConnection<Sqlite>,
) -> Result<()> {
    // self.update_item(item.clone(), index)?;

    // debug!(?item);
    let verse_order =
        ron::ser::to_string(&item.verses).into_diagnostic()?;

    let audio = item
        .audio
        .map(|a| a.to_str().unwrap_or_default().to_string());

    let background = item
        .background
        .map(|b| b.path.to_str().unwrap_or_default().to_string());

    let lyrics =
        ron::ser::to_string(&item.verse_map).into_diagnostic()?;

    // let text_alignment = item.text_alignment.map(|ta| match ta {
    //     TextAlignment::TopLeft => todo!(),
    //     TextAlignment::TopCenter => todo!(),
    //     TextAlignment::TopRight => todo!(),
    //     TextAlignment::MiddleLeft => todo!(),
    //     TextAlignment::MiddleCenter => todo!(),
    //     TextAlignment::MiddleRight => todo!(),
    //     TextAlignment::BottomLeft => todo!(),
    //     TextAlignment::BottomCenter => todo!(),
    //     TextAlignment::BottomRight => todo!(),
    // })

    query!(
        r#"UPDATE songs SET title = $2, lyrics = $3, author = $4, ccli = $5, verse_order = $6, audio = $7, font = $8, font_size = $9, background = $10 WHERE id = $1"#,
        item.id,
        item.title,
        lyrics,
        item.author,
        item.ccli,
        verse_order,
        audio,
        item.font,
        item.font_size,
        background
    )
        .execute(&mut db.detach())
        .await
        .into_diagnostic()?;

    Ok(())
}

impl Song {
    #[must_use]
    pub fn get_lyric(&self, verse: &VerseName) -> Option<String> {
        self.verse_map.as_ref().and_then(|verse_map| {
            verse_map
                .get(verse)
                .cloned()
                .map(|lyric| lyric.trim_end().to_string())
        })
    }

    pub fn set_lyrics<T: Into<String>>(
        &mut self,
        verse: &VerseName,
        lyrics: T,
    ) {
        let lyric_copy = lyrics.into();
        if let Some(verse_map) = self.verse_map.as_mut() {
            debug!(?verse_map, "should update");
            verse_map
                .entry(*verse)
                .and_modify(|old_lyrics| {
                    *old_lyrics = lyric_copy.clone()
                })
                .or_insert(lyric_copy);
            debug!(?verse_map, "should be updated");
        } else {
            debug!(?self.verse_map, "should create");
            let mut verse_map = HashMap::new();
            verse_map
                .entry(*verse)
                .and_modify(|old_lyrics| {
                    *old_lyrics = lyric_copy.clone()
                })
                .or_insert(lyric_copy);
            self.verse_map = Some(verse_map);
        }
        debug!(?self.verse_map);
    }

    pub fn get_lyrics(&self) -> Result<Vec<String>> {
        // ---------------------------------
        // new implementation
        // ---------------------------------

        if let Some(verses) = self.verses.as_ref() {
            let mut lyrics = vec![];
            for verse in verses {
                if verse == &VerseName::Blank {
                    lyrics.push("".into());
                    continue;
                }
                if let Some(lyric) = self.get_lyric(verse) {
                    lyrics.push(lyric)
                }
            }
            return Ok(lyrics);
        } else {
            return Err(miette!("No verses in this song yet"));
        }

        // ---------------------------------
        // old implementation
        // ---------------------------------

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
            let verse_order = self.verses.clone();

            let mut lyric_map = HashMap::new();
            let mut verse_title = String::new();
            let mut lyric = String::new();
            for (i, line) in raw_lyrics.split('\n').enumerate() {
                if VERSE_KEYWORDS.contains(&line) {
                    if i != 0 {
                        lyric_map.insert(verse_title, lyric);
                        lyric = String::new();
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
                let verse_name = &verse.get_name();
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
                    lyric_list.push(lyric.clone());
                } else {
                    // error!("NOT WORKING!");
                }
            }
            // for lyric in lyric_list.iter() {
            //     debug!(lyric = ?lyric)
            // }
            Ok(lyric_list)
        } else {
            Err(miette!("There are no lyrics"))
        }
    }

    // TODO update_verse needs to also change the lyrics for the song such that
    // the song can be sent to the db and it's lyrics will actually change. Or we
    // could have the update_song_in_db function recreate the lyrics from the new
    // verse layout. But I do feel like it belongs here more.
    pub fn update_verse(
        &mut self,
        index: usize,
        verse: VerseName,
        lyric: String,
    ) {
        debug!(index, ?verse, lyric);
        self.set_lyrics(&verse, lyric);
        if let Some(verses) = self.verses.as_mut()
            && let Some(old_verse) = verses.get_mut(index)
        {
            debug!(?old_verse, ?verse);
            *old_verse = verse;
        }

        if let Some(verses) = &self.verses {
            let mut new_lyrics = String::new();

            for verse in verses {
                let Some(lyrics) = self.get_lyric(verse) else {
                    return;
                };

                let verse_name = match verse {
                    VerseName::Verse { number } => {
                        format!("Verse {number}")
                    }
                    VerseName::PreChorus { number } => {
                        format!("Pre-Chorus {number}")
                    }
                    VerseName::Chorus { number } => {
                        format!("Chorus {number}")
                    }
                    VerseName::PostChorus { number } => {
                        format!("Post-Chorus {number}")
                    }
                    VerseName::Bridge { number } => {
                        format!("Bridge {number}")
                    }
                    VerseName::Intro { number } => {
                        format!("Intro {number}")
                    }
                    VerseName::Outro { number } => {
                        format!("Outro {number}")
                    }
                    VerseName::Instrumental { number } => {
                        format!("Instrumental {number}")
                    }
                    VerseName::Other { number } => {
                        format!("Other {number}")
                    }
                    VerseName::Blank => "Blank".into(),
                };

                new_lyrics.push_str(&verse_name);
                new_lyrics.push('\n');
                new_lyrics.push_str(&lyrics);
                new_lyrics.push('\n');
                new_lyrics.push('\n');
            }

            debug!(
                same = self.lyrics == Some(new_lyrics.clone()),
                old_lyrics = self.lyrics,
                new_lyrics
            );
            self.lyrics = Some(new_lyrics);
        }
    }

    pub fn get_next_verse_name(&self) -> VerseName {
        if let Some(verse_names) = &self.verses {
            let verses: Vec<&VerseName> = verse_names
                .iter()
                .filter(|verse| match verse {
                    VerseName::Verse { .. } => true,
                    _ => false,
                })
                .sorted()
                .collect();
            let choruses: Vec<&VerseName> = verse_names
                .iter()
                .filter(|verse| match verse {
                    VerseName::Chorus { .. } => true,
                    _ => false,
                })
                .collect();
            let bridges: Vec<&VerseName> = verse_names
                .iter()
                .filter(|verse| match verse {
                    VerseName::Bridge { .. } => true,
                    _ => false,
                })
                .collect();
            if verses.is_empty() {
                VerseName::Verse { number: 1 }
            } else if choruses.is_empty() {
                VerseName::Chorus { number: 1 }
            } else if verses.len() == 1 {
                let verse_number =
                    if let Some(last_verse) = verses.iter().last() {
                        match last_verse {
                            VerseName::Verse { number } => *number,
                            _ => 0,
                        }
                    } else {
                        0
                    };
                if verse_number > 1 {
                    return VerseName::Verse { number: 1 };
                }
                VerseName::Verse { number: 2 }
            } else if bridges.is_empty() {
                VerseName::Bridge { number: 1 }
            } else {
                if let Some(last_verse) = verses.iter().last()
                    && let VerseName::Verse { number } = last_verse
                {
                    return VerseName::Verse { number: number + 1 };
                }
                VerseName::Verse { number: 1 }
            }
        } else {
            VerseName::Verse { number: 1 }
        }
    }

    pub fn add_verse(
        &mut self,
        verse: VerseName,
        lyric: impl Into<String>,
    ) {
        let lyric: String = lyric.into();
        self.set_lyrics(&verse, lyric);
        if let Some(verses) = self.verses.as_mut() {
            verses.push(verse);
        } else {
            self.verses = Some(vec![verse]);
        };
    }

    pub(crate) fn verse_name_from_str(
        &self,
        verse_name: String,        // chorus 2
        old_verse_name: VerseName, // v4
    ) -> VerseName {
        if old_verse_name.get_name() == verse_name {
            return old_verse_name;
        };
        if let Some(verses) =
            self.verse_map.clone().map(|verse_map| {
                verse_map.into_keys().collect::<Vec<VerseName>>()
            })
        {
            verses
                .into_iter()
                .filter(|verse| {
                    verse
                        .get_name()
                        .split_whitespace()
                        .next()
                        .unwrap()
                        == &verse_name
                })
                .sorted()
                .last()
                .map_or_else(
                    || VerseName::from_string(verse_name),
                    |verse_name| verse_name.next(),
                )
        } else {
            VerseName::from_string(verse_name)
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    pub fn test_song_lyrics() {
        let mut song = Song::default();
        let mut map = HashMap::new();
        map.insert(
            VerseName::Verse { number: 1 },
            "When You found me,
I was so blind
My sin was before me,
I was swallowed by pride"
                .into(),
        );
        map.insert(
            VerseName::Chorus { number: 1 },
            "But out of the darkness,
You brought me to Your light
You showed me new mercy
And opened up my eyes"
                .into(),
        );
        map.insert(
            VerseName::Chorus { number: 2 },
            "From the day
You saved my soul
'Til the very moment
When I come home

I'll sing, I'll dance,
My heart will overflow
From the day
You saved my soul"
                .into(),
        );
        map.insert(
            VerseName::Verse { number: 2 },
            "Where brilliant light
Is all around
And endless joy
Is the only sound"
                .into(),
        );
        map.insert(
            VerseName::Chorus { number: 3 },
            "Oh, rest my heart
Forever now
Oh, in Your arms
I'll always be found"
                .into(),
        );
        map.insert(
            VerseName::Other { number: 1 },
            "From the Day
I Am They"
                .into(),
        );
        map.insert(
            VerseName::Bridge { number: 1 },
            "My love is Yours
My heart is Yours
My life is Yours
Forever

My love is Yours
My heart is Yours
My life is Yours
Forever"
                .into(),
        );
        map.insert(
            VerseName::Outro { number: 1 },
            "Oh Oh Oh
From the day
You saved my soul"
                .into(),
        );
        map.insert(VerseName::Blank, "".into());
        song.verse_map = Some(map);
        song.verses = Some(vec![
            VerseName::Other { number: 1 },
            VerseName::Verse { number: 1 },
            VerseName::Chorus { number: 1 },
            VerseName::Chorus { number: 2 },
            VerseName::Blank,
            VerseName::Verse { number: 2 },
            VerseName::Chorus { number: 3 },
            VerseName::Chorus { number: 2 },
            VerseName::Blank,
            VerseName::Bridge { number: 1 },
            VerseName::Chorus { number: 2 },
            VerseName::Chorus { number: 2 },
            VerseName::Outro { number: 1 },
            VerseName::Blank,
        ]);
        song.verse_order =
            "O1 V1 C1 C2 O2 V2 C3 C2 O2 B1 C2 C2 E1 O2"
                .to_string()
                .split(' ')
                .map(|s| Some(s.to_string()))
                .collect();
        let lyrics = song.get_lyrics();
        match lyrics {
            Ok(lyrics) => {
                assert_eq!(
                    vec![
                        "From the Day\nI Am They",
                        "When You found me,\nI was so blind\nMy sin was before me,\nI was swallowed by pride",
                        "But out of the darkness,\nYou brought me to Your light\nYou showed me new mercy\nAnd opened up my eyes",
                        "From the day\nYou saved my soul\n'Til the very moment\nWhen I come home\n\nI'll sing, I'll dance,\nMy heart will overflow\nFrom the day\nYou saved my soul",
                        "",
                        "Where brilliant light\nIs all around\nAnd endless joy\nIs the only sound",
                        "Oh, rest my heart\nForever now\nOh, in Your arms\nI'll always be found",
                        "From the day\nYou saved my soul\n'Til the very moment\nWhen I come home\n\nI'll sing, I'll dance,\nMy heart will overflow\nFrom the day\nYou saved my soul",
                        "",
                        "My love is Yours\nMy heart is Yours\nMy life is Yours\nForever\n\nMy love is Yours\nMy heart is Yours\nMy life is Yours\nForever",
                        "From the day\nYou saved my soul\n'Til the very moment\nWhen I come home\n\nI'll sing, I'll dance,\nMy heart will overflow\nFrom the day\nYou saved my soul",
                        "From the day\nYou saved my soul\n'Til the very moment\nWhen I come home\n\nI'll sing, I'll dance,\nMy heart will overflow\nFrom the day\nYou saved my soul",
                        "Oh Oh Oh\nFrom the day\nYou saved my soul",
                        ""
                    ],
                    lyrics
                );
            }
            Err(e) => {
                assert!(false, "{:?}", e)
            }
        }
    }

    async fn model() -> Model<Song> {
        let song_model: Model<Song> = Model {
            items: vec![],
            kind: LibraryKind::Song,
            // db: crate::core::model::get_db().await,
        };
        song_model
    }

    async fn add_db() -> Result<SqlitePool> {
        let mut data = dirs::data_local_dir().unwrap();
        data.push("lumina");
        data.push("library-db.sqlite3");
        let mut db_url = String::from("sqlite://");
        db_url.push_str(data.to_str().unwrap());
        SqlitePool::connect(&db_url).await.into_diagnostic()
    }

    #[tokio::test]
    async fn test_db_and_model() {
        let mut db = add_db().await.unwrap();
        let mut song_model = model().await;
        song_model.load_from_db(&mut db).await;
        if let Some(song) = song_model.find(|s| s.id == 7) {
            let test_song = test_song();
            if let Ok(song_lyrics) = song.get_lyrics()
                && let Ok(test_lyrics) = test_song.get_lyrics()
            {
                assert_eq!(song_lyrics, test_lyrics)
            } else {
                assert!(false, "lyrics aren't retrieving")
            }
        } else {
            dbg!(song_model);
            assert!(false);
        }
    }

    #[test]
    fn test_song_slide_speed() {
        let song = test_song();
        let slides = song.to_slides();
        if let Ok(slides) = slides {
            assert!(true, "{:?}", slides);
        } else {
            assert!(false, "Slides failed");
        }
    }

    #[tokio::test]
    async fn test_song_from_db() {
        let song = test_song();
        let mut db = crate::core::model::get_db().await;
        let result = get_song_from_db(7, &mut db).await;
        match result {
            Ok(db_song) => {
                if let Ok(song_lyrics) = song.get_lyrics()
                    && let Ok(db_lyrics) = db_song.get_lyrics()
                {
                    assert_eq!(song_lyrics, db_lyrics)
                } else {
                    assert!(false, "lyrics aren't retrieving")
                }
            }
            Err(e) => assert!(false, "{e}"),
        }
    }

    #[tokio::test]
    async fn test_update() {
        let mut db = add_db().await.unwrap();
        let song = test_song();
        let cloned_song = song.clone();
        let mut song_model: Model<Song> = model().await;
        song_model.load_from_db(&mut db).await;

        match song_model.update_item(song, 2) {
            Ok(()) => assert_eq!(
                &cloned_song,
                song_model.find(|s| s.id == 7).unwrap()
            ),
            Err(e) => assert!(false, "{e}"),
        }
    }

    fn test_song() -> Song {
        let lyrics = "Some({Verse(number:4):\"Our Savior displayed\\nOn a criminal\\'s cross\\n\\nDarkness rejoiced as though\\nHeaven had lost\\n\\nBut then Jesus arose\\nWith our freedom in hand\\n\\nThat\\'s when death was arrested\\nAnd my life began\\n\\nThat\\'s when death was arrested\\nAnd my life began\",Intro(number:1):\"Death Was Arrested\\nNorth Point Worship\",Verse(number:3):\"Released from my chains,\\nI\\'m a prisoner no more\\n\\nMy shame was a ransom\\nHe faithfully bore\\n\\nHe cancelled my debt and\\nHe called me His friend\\n\\nWhen death was arrested\\nAnd my life began\",Bridge(number:1):\"Oh, we\\'re free, free,\\nForever we\\'re free\\n\\nCome join the song\\nOf all the redeemed\\n\\nYes, we\\'re free, free,\\nForever amen\\n\\nWhen death was arrested\\nAnd my life began\\n\\nOh, we\\'re free, free,\\nForever we\\'re free\\n\\nCome join the song\\nOf all the redeemed\\n\\nYes, we\\'re free, free,\\nForever amen\\n\\nWhen death was arrested\\nAnd my life began\",Other(number:99):\"When death was arrested\\nAnd my life began\\n\\nThat\\'s when death was arrested\\nAnd my life began\",Verse(number:2):\"Ash was redeemed\\nOnly beauty remains\\n\\nMy orphan heart\\nWas given a name\\n\\nMy mourning grew quiet,\\nMy feet rose to dance\\n\\nWhen death was arrested\\nAnd my life began\",Verse(number:1):\"Alone in my sorrow\\nAnd dead in my sin\\n\\nLost without hope\\nWith no place to begin\\n\\nYour love made a way\\nTo let mercy come in\\n\\nWhen death was arrested\\nAnd my life began\",Chorus(number:1):\"Oh, Your grace so free,\\nWashes over me\\n\\nYou have made me new,\\nNow life begins with You\\n\\nIt\\'s Your endless love,\\nPouring down on us\\n\\nYou have made us new,\\nNow life begins with You\"})".to_string();
        let verse_map: Option<HashMap<VerseName, String>> =
            ron::from_str(&lyrics).unwrap();
        Song {
            id: 7,
            title: "Death Was Arrested".to_string(),
            lyrics: Some(lyrics),
            author: Some(
                "North Point Worship".to_string(),
            ),
            ccli: None,
            audio: Some("file:///home/chris/music/North Point InsideOut/Nothing Ordinary, Pt. 1 (Live)/05 Death Was Arrested (feat. Seth Condrey).mp3".into()),
            verse_order: Some(vec!["Some([Chorus(number:1),Intro(number:1),Other(number:99),Bridge(number:1),Verse(number:4),Verse(number:2),Verse(number:3),Verse(number:1)])".to_string()]),
            background: Some(Background::try_from("file:///home/chris/nc/tfc/openlp/Flood/motions/Ocean_Floor_HD.mp4").unwrap()),
            text_alignment: Some(TextAlignment::MiddleCenter),
            font: Some("Quicksand Bold".to_string()),
            font_size: Some(80),
            stroke_size: None,
            verses: Some(vec![VerseName::Chorus { number: 1 }, VerseName::Intro { number: 1 }, VerseName::Other { number: 99 }, VerseName::Bridge { number: 1 }, VerseName::Verse { number: 4 }, VerseName::Verse { number: 2 }, VerseName::Verse { number: 3 }, VerseName::Verse { number: 1 }
            ]),
            verse_map,
            ..Default::default()
        }
    }

    fn test_lisp_song() -> Value {
        let lisp = read_to_string("./test_song.lisp").expect("oops");
        let lisp_value = crisp::reader::read(&lisp);
        match lisp_value {
            Value::List(v) => v.first().unwrap().clone(),
            _ => Value::Nil,
        }
    }

    #[test]
    fn test_verse_names_and_adding() {
        let mut song = Song::default();
        song.verses = Some(vec![]);
        let name = song.get_next_verse_name();
        assert_eq!(name, VerseName::Verse { number: 1 });
        song.verses = Some(vec![VerseName::Verse { number: 1 }]);
        let name = song.get_next_verse_name();
        assert_eq!(name, VerseName::Chorus { number: 1 });
        song.verses = Some(vec![
            VerseName::Verse { number: 1 },
            VerseName::Chorus { number: 1 },
        ]);
        let name = song.get_next_verse_name();
        assert_eq!(name, VerseName::Verse { number: 2 });
        song.verses = Some(vec![
            VerseName::Chorus { number: 1 },
            VerseName::Verse { number: 2 },
        ]);
        let name = song.get_next_verse_name();
        assert_eq!(name, VerseName::Verse { number: 1 });
        song.verses = Some(vec![
            VerseName::Verse { number: 1 },
            VerseName::Chorus { number: 1 },
            VerseName::Verse { number: 2 },
        ]);
        let name = song.get_next_verse_name();
        assert_eq!(name, VerseName::Bridge { number: 1 });
        song.verses = Some(vec![
            VerseName::Verse { number: 1 },
            VerseName::Chorus { number: 1 },
            VerseName::Verse { number: 2 },
            VerseName::Bridge { number: 1 },
        ]);
        let name = song.get_next_verse_name();
        assert_eq!(name, VerseName::Verse { number: 3 });
        song.verses = Some(vec![
            VerseName::Verse { number: 1 },
            VerseName::Chorus { number: 1 },
            VerseName::Verse { number: 2 },
            VerseName::Verse { number: 3 },
            VerseName::Bridge { number: 1 },
        ]);
        let name = song.get_next_verse_name();
        assert_eq!(name, VerseName::Verse { number: 4 });
        song.add_verse(VerseName::Verse { number: 4 }, "");
        let name = song.get_next_verse_name();
        assert_eq!(name, VerseName::Verse { number: 5 });
    }

    // #[test]
    // pub fn test_lisp_conversion() {
    //     let value = test_lisp_song();
    //     let lisp_song = Song::from(value);
    //     let test_song = test_song();
    //     assert_eq!(test_song, lisp_song);
    // }
}
