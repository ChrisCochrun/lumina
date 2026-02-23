use std::{
    borrow::Cow, collections::HashMap, option::Option, path::PathBuf,
};

use cosmic::{
    cosmic_theme::palette::Srgb,
    iced::{
        clipboard::mime::AsMimeTypes,
        font::{Style, Weight},
    },
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

use crate::{
    Slide, SlideBuilder,
    core::{
        content::Content,
        kinds::ServiceItemKind,
        model::{LibraryKind, Model},
        service_items::ServiceTrait,
        slide::{self, Background, TextAlignment},
        song_search::OnlineSong,
    },
    ui::text_svg::{Color, Font, Stroke, shadow, stroke},
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
    pub font_weight: Option<Weight>,
    pub font_style: Option<Style>,
    pub text_color: Option<Srgb>,
    pub stroke_size: Option<u16>,
    pub stroke_color: Option<Srgb>,
    pub shadow_size: Option<u16>,
    pub shadow_offset: Option<(i16, i16)>,
    pub shadow_color: Option<Srgb>,
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
    #[must_use]
    pub fn from_string(name: &str) -> Self {
        match name {
            "Verse" => Self::Verse { number: 1 },
            "Pre-Chorus" => Self::PreChorus { number: 1 },
            "Chorus" => Self::Chorus { number: 1 },
            "Post-Chorus" => Self::PostChorus { number: 1 },
            "Bridge" => Self::Bridge { number: 1 },
            "Intro" => Self::Intro { number: 1 },
            "Outro" => Self::Outro { number: 1 },
            "Instrumental" => Self::Instrumental { number: 1 },
            "Other" => Self::Other { number: 1 },
            // Blank is included in wildcard
            _ => Self::Blank,
        }
    }

    #[must_use]
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

    #[must_use]
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
        self.author
            .clone()
            .unwrap_or_else(|| "Author missing".into())
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
        let lyrics: Vec<String> = self
            .verses
            .as_ref()
            .ok_or_else(|| {
                miette!("There are no verses assigned yet.")
            })?
            .iter()
            .filter_map(|verse| self.get_lyric(verse))
            .flat_map(|lyric| {
                lyric
                    .split("\n\n")
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<String>>()
            })
            .collect();

        debug!(?lyrics);
        let slides: Vec<Slide> = lyrics
            .iter()
            .filter_map(|l| {
                let font = Font::default()
                    .name(
                        self.font
                            .clone()
                            .unwrap_or_else(|| "Calibri".into()),
                    )
                    .style(self.font_style.unwrap_or_default())
                    .weight(self.font_weight.unwrap_or_default())
                    .size(
                        u8::try_from(self.font_size.unwrap_or(100))
                            .unwrap_or(100),
                    );
                let stroke_size =
                    self.stroke_size.unwrap_or_default();
                let stroke: Stroke = stroke(
                    stroke_size,
                    self.stroke_color
                        .map(Color::from)
                        .unwrap_or_default(),
                );
                let shadow_size =
                    self.shadow_size.unwrap_or_default();
                let shadow = shadow(
                    self.shadow_offset.unwrap_or_default().0,
                    self.shadow_offset.unwrap_or_default().1,
                    shadow_size,
                    self.shadow_color
                        .map(Color::from)
                        .unwrap_or_default(),
                );
                let builder = SlideBuilder::new();
                let builder = if shadow_size > 0 {
                    builder.shadow(shadow)
                } else {
                    builder
                };
                let builder = if stroke_size > 0 {
                    builder.stroke(stroke)
                } else {
                    builder
                };
                builder
                    .background(
                        self.background.clone().unwrap_or_default(),
                    )
                    .font(font)
                    .font_size(self.font_size.unwrap_or_default())
                    .text_alignment(
                        self.text_alignment.unwrap_or_default(),
                    )
                    .text_color(
                        self.text_color.unwrap_or_else(|| {
                            Srgb::new(1.0, 1.0, 1.0)
                        }),
                    )
                    .audio(self.audio.clone().unwrap_or_default())
                    .video_loop(true)
                    .video_start_time(0.0)
                    .video_end_time(0.0)
                    .text(l)
                    .build()
                    .ok()
            })
            .collect();

        Ok(slides)
    }

    fn box_clone(&self) -> Box<dyn ServiceTrait> {
        Box::new((*self).clone())
    }
}

#[allow(clippy::too_many_lines)]
impl FromRow<'_, SqliteRow> for Song {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let lyrics: &str = row.try_get("lyrics")?;

        let Ok(verse_map) = ron::de::from_str::<
            Option<HashMap<VerseName, String>>,
        >(lyrics) else {
            return Err(sqlx::Error::ColumnDecode {
                index: "8".into(),
                source: miette!(
                    "Couldn't decode the song into verses"
                )
                .into(),
            });
        };
        let verse_order: &str = row.try_get("verse_order")?;
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

        let stroke_size = match row.try_get("stroke_size") {
            Ok(size) => Some(size),
            Err(e) => {
                error!(?e);
                None
            }
        };
        let stroke_color = row
            .try_get("stroke_color")
            .ok()
            .and_then(|color: String| {
                ron::de::from_str::<Option<Srgb>>(&color).ok()
            })
            .flatten();
        let shadow_size = row.try_get("shadow_size").ok();
        let shadow_color = row
            .try_get("shadow_color")
            .ok()
            .and_then(|color: String| {
                ron::de::from_str::<Option<Srgb>>(&color).ok()
            })
            .flatten();
        let shadow_offset = match (
            row.try_get("shadow_offset_x").ok(),
            row.try_get("shadow_offset_y").ok(),
        ) {
            (Some(x), Some(y)) => Some((x, y)),
            _ => None,
        };

        let style_string: String = row.try_get("style")?;
        let font_style =
            ron::de::from_str::<Option<Style>>(&style_string)
                .ok()
                .flatten();

        let weight_string: String = row.try_get("weight")?;
        let font_weight =
            ron::de::from_str::<Option<Weight>>(&weight_string)
                .ok()
                .flatten();

        Ok(Self {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            lyrics: Some(lyrics.to_string()),
            author: row.try_get("author")?,
            ccli: row.try_get("ccli")?,
            audio: Some(PathBuf::from({
                let string: String = row.try_get("audio")?;
                string
            })),
            verse_order: Some(verse_order),
            background: {
                let string: String = row.try_get("background")?;
                Background::try_from(string).ok()
            },
            text_alignment: Some({
                let horizontal_alignment: String =
                    row.try_get("horizontal_text_alignment")?;
                let vertical_alignment: String =
                    row.try_get("vertical_text_alignment")?;
                // debug!(horizontal_alignment, vertical_alignment);
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
            font: row.try_get("font")?,
            font_size: row.try_get("font_size")?,
            font_style,
            font_weight,
            stroke_size,
            stroke_color,
            shadow_size,
            shadow_color,
            shadow_offset,
            verses,
            verse_map,
            ..Default::default()
        })
    }
}

impl From<OnlineSong> for Song {
    fn from(value: OnlineSong) -> Self {
        let mut song = Self::default();
        song.verse_map = Some(HashMap::new());
        for line in value.lyrics.lines() {
            let next_verse = song.get_next_verse_name();
            if let Some(verse_map) = song.verse_map.as_mut() {
                verse_map
                    .entry(next_verse)
                    .or_insert_with(|| line.to_string());
            }
            if let Some(verses) = song.verses.as_mut() {
                verses.push(next_verse);
            } else {
                song.verses = Some(vec![next_verse]);
            }
        }
        song
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

#[allow(clippy::option_if_let_else)]
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_lines)]
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
        list.get(pos)
            .map_or_else(|| String::from("song"), String::from)
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
                } && matches!(&inner[1], Value::String(_)))
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
            "i2" => r"\n\nIntro 2\n",
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
    let row = query("SELECT verse_order, font_size, background_type, horizontal_text_alignment, vertical_text_alignment, title, font, background, lyrics, ccli, author, audio, stroke_size, stroke_color, shadow_color, shadow_size, shadow_offset_x, shadow_offset_y, style, weight, id from songs where id = $1").bind(index).fetch_one(db).await.into_diagnostic()?;
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
        let db1 = db.acquire().await.expect("Database not found");
        let result = query("SELECT verse_order, font_size, background_type, horizontal_text_alignment, vertical_text_alignment, title, font, background, lyrics, ccli, author, audio, stroke_size, shadow_size, stroke_color, shadow_color, shadow_offset_x, shadow_offset_y, style, weight, id from songs").fetch_all(&mut db1.detach()).await;
        match result {
            Ok(s) => {
                for song in s {
                    // let db2 = db.acquire().await.unwrap();
                    match Song::from_row(&song) {
                        Ok(song) => {
                            let _ = self.add_item(song);
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
        song.verse_order.clone().map_or_else(String::new, |vo| {
            vo.into_iter()
                .map(|mut s| {
                    s.push(' ');
                    s
                })
                .collect::<String>()
        })
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
    song.id = i32::try_from(res.last_insert_rowid()).expect(
        "Fairly confident that this number won't get that high",
    );
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

    let lyrics = item.verse_map.map(|map| {
        map.iter()
            .map(|(name, lyric)| {
                let lyric = lyric.trim_end_matches('\n').to_string();
                (name.to_owned(), lyric)
            })
            .collect::<HashMap<VerseName, String>>()
    });
    let lyrics = ron::ser::to_string(&lyrics).into_diagnostic()?;

    let (vertical_alignment, horizontal_alignment) =
        item.text_alignment.map_or_else(
            || ("center", "center"),
            |ta| match ta {
                TextAlignment::TopLeft => ("top", "left"),
                TextAlignment::TopCenter => ("top", "center"),
                TextAlignment::TopRight => ("top", "right"),
                TextAlignment::MiddleLeft => ("center", "left"),
                TextAlignment::MiddleCenter => ("center", "center"),
                TextAlignment::MiddleRight => ("center", "right"),
                TextAlignment::BottomLeft => ("bottom", "left"),
                TextAlignment::BottomCenter => ("bottom", "center"),
                TextAlignment::BottomRight => ("bottom", "right"),
            },
        );

    let stroke_size = item.stroke_size.unwrap_or_default();
    let shadow_size = item.shadow_size.unwrap_or_default();
    let (shadow_offset_x, shadow_offset_y) =
        item.shadow_offset.unwrap_or_default();

    let stroke_color =
        ron::ser::to_string(&item.stroke_color).into_diagnostic()?;
    let shadow_color =
        ron::ser::to_string(&item.shadow_color).into_diagnostic()?;

    let style =
        ron::ser::to_string(&item.font_style).into_diagnostic()?;
    let weight =
        ron::ser::to_string(&item.font_weight).into_diagnostic()?;

    // debug!(
    //     ?stroke_size,
    //     ?stroke_color,
    //     ?shadow_size,
    //     ?shadow_color,
    //     ?shadow_offset_x,
    //     ?shadow_offset_y
    // );

    let result = query!(
        r#"UPDATE songs SET title = $2, lyrics = $3, author = $4, ccli = $5, verse_order = $6, audio = $7, font = $8, font_size = $9, background = $10, horizontal_text_alignment = $11, vertical_text_alignment = $12, stroke_color = $13, shadow_color = $14, stroke_size = $15, shadow_size = $16, shadow_offset_x = $17, shadow_offset_y = $18, style = $19, weight = $20 WHERE id = $1"#,
        item.id,
        item.title,
        lyrics,
        item.author,
        item.ccli,
        verse_order,
        audio,
        item.font,
        item.font_size,
        background,
        horizontal_alignment,
        vertical_alignment,
        stroke_color,
        shadow_color,
        stroke_size,
        shadow_size,
        shadow_offset_x,
        shadow_offset_y,
        style,
        weight
    )
        .execute(&mut db.detach())
        .await
        .into_diagnostic()?;

    debug!(rows_affected = ?result.rows_affected());

    Ok(())
}

impl Song {
    #[must_use]
    pub fn get_lyric(&self, verse: &VerseName) -> Option<String> {
        self.verse_map.as_ref().and_then(|verse_map| {
            verse_map.get(verse).cloned().map(|lyric| {
                lyric.trim().trim_end_matches('\n').to_string()
            })
        })
    }

    pub fn set_lyrics<T: Into<String>>(
        &mut self,
        verse: &VerseName,
        lyrics: T,
    ) {
        let lyric_copy = lyrics.into().trim().to_string();
        if let Some(verse_map) = self.verse_map.as_mut() {
            // debug!(?verse_map, "should update");
            verse_map
                .entry(*verse)
                .and_modify(|old_lyrics| {
                    old_lyrics.clone_from(&lyric_copy);
                })
                .or_insert(lyric_copy);
            // debug!(?verse_map, "should be updated");
        } else {
            // debug!(?self.verse_map, "should create");
            let mut verse_map = HashMap::new();
            verse_map.insert(*verse, lyric_copy);
            self.verse_map = Some(verse_map);
        }
        // debug!(?self.verse_map);
    }

    pub fn get_lyrics(&self) -> Result<Vec<String>> {
        if let Some(verses) = self.verses.as_ref() {
            let mut lyrics = vec![];
            for verse in verses {
                if verse == &VerseName::Blank {
                    lyrics.push(String::new());
                    continue;
                }
                if let Some(lyric) = self.get_lyric(verse) {
                    lyrics.push(lyric);
                }
            }
            return Ok(lyrics);
        }
        Err(miette!("No verses in this song yet"))
    }

    pub fn update_verse_name(
        &mut self,
        verse: VerseName,
        old_verse: &VerseName,
    ) {
        if let Some(verse_map) = self.verse_map.as_mut()
            && let Some(lyric) = verse_map.remove(old_verse)
        {
            if verse == VerseName::Blank {
                verse_map.insert(verse, String::new());
            } else {
                verse_map.insert(verse, lyric);
            }
        }
        let Some(verses) = self.verses.clone() else {
            return;
        };
        let mut new_verses: Vec<VerseName> = verses
            .into_iter()
            .filter(|verse| verse != old_verse)
            .collect();
        new_verses.push(verse);
        self.verses = Some(new_verses);
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

    #[must_use]
    pub fn get_next_verse_name(&self) -> VerseName {
        if let Some(verse_names) = &self.verses {
            let verses = verse_names
                .iter()
                .filter(|verse| {
                    matches!(verse, VerseName::Verse { .. })
                })
                .sorted();
            let mut choruses = verse_names.iter().filter(|verse| {
                matches!(verse, VerseName::Chorus { .. })
            });
            let mut bridges = verse_names.iter().filter(|verse| {
                matches!(verse, VerseName::Bridge { .. })
            });
            if verses.len() == 0 {
                VerseName::Verse { number: 1 }
            } else if choruses.next().is_none() {
                VerseName::Chorus { number: 1 }
            } else if verses.len() == 1 {
                let verse_number =
                    if let Some(VerseName::Verse { number }) =
                        verses.last()
                    {
                        *number
                    } else {
                        0
                    };
                if verse_number > 1 {
                    return VerseName::Verse { number: 1 };
                }
                VerseName::Verse { number: 2 }
            } else if bridges.next().is_none() {
                VerseName::Bridge { number: 1 }
            } else {
                if let Some(last_verse) = verses.last()
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
        }
    }

    pub(crate) fn verse_name_from_str(
        &self,
        verse_name: &str,          // chorus 2
        old_verse_name: VerseName, // v4
    ) -> VerseName {
        if old_verse_name.get_name() == verse_name {
            return old_verse_name;
        }
        self.verse_map.clone().map(|verse_map| {
                verse_map.into_keys().collect::<Vec<VerseName>>()
            }).map_or_else(|| VerseName::from_string(verse_name), |verses| {
            verses
                .into_iter()
                .filter(|verse| {
                    verse
                        .get_name()
                        .split_whitespace()
                        .next()
                        .expect("Shouldn't fail, the get_name() fn won't return a string that is blank or all whitespace")
                        == verse_name
                })
                .sorted()
                .last()
                .map_or_else(
                    || VerseName::from_string(verse_name),
                    |verse_name| verse_name.next(),
                )
        })
    }

    pub(crate) fn delete_verse(&mut self, verse: VerseName) {
        if let Some(verses) = self.verses.as_mut() {
            verses.retain(|inner| inner != &verse);
        }
        if let Some(map) = self.verse_map.as_mut() {
            let _ = map.remove(&verse);
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::ui::text_svg::text_svg_generator_with_cache;

    use super::*;
    use pretty_assertions::assert_eq;
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    use resvg::usvg::fontdb;

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
        let db_url = String::from("sqlite://./test.db");
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
        let mut db = add_db().await.unwrap().acquire().await.unwrap();
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

    pub fn test_song() -> Song {
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

    #[tokio::test]
    async fn test_song_to_slide() {
        let song = test_song();
        let songs: Vec<Song> = (0..100)
            .map(|index| {
                let mut song = song.clone();
                song.id = index;
                song
            })
            .collect();
        let fontdb = Arc::new(fontdb::Database::new());
        songs.into_par_iter().for_each(|song| {
            let slides = song.to_slides().unwrap();
            slides.into_par_iter().for_each(|slide| {
                text_svg_generator_with_cache(slide, &fontdb, None)
                    .map_or_else(
                        |e| assert!(false, "{e}"),
                        |slide| {
                            assert!(slide.text_svg.is_some_and(
                                |svg| svg.handle.is_some()
                            ))
                        },
                    )
            });
        });
    }

    // extern crate test;
    // use test::{Bencher, black_box};

    // #[bench]
    // fn bench_pow(b: &mut Bencher) {
    //     // Optionally include some setup
    //     let x: f64 = 211.0 * 11.0;
    //     let y: f64 = 301.0 * 103.0;

    //     b.iter(|| {
    //         // Inner closure, the actual test
    //         for i in 1..100 {
    //             black_box(x.powf(y).powf(x));
    //         }
    //     });
    // }

    // #[test]
    // pub fn test_lisp_conversion() {
    //     let value = test_lisp_song();
    //     let lisp_song = Song::from(value);
    //     let test_song = test_song();
    //     assert_eq!(test_song, lisp_song);
    // }
}
