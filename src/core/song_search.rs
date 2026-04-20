use crate::core::songs::{Song, VerseName};
use itertools::Itertools;
use miette::{IntoDiagnostic, Result, miette};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_till1, take_until};
use nom::character::complete::{digit0, space0};
use nom::combinator::rest;
use nom::multi::many1;
use nom::sequence::{delimited, pair};
use nom::{IResult, Parser};
use reqwest::header;
use scraper::Element;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Display;
use tracing::{debug, error};

#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
)]
pub struct OnlineSong {
    pub lyrics: String,
    pub title: String,
    pub author: String,
    pub provider: Provider,
    pub link: String,
}

#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
)]
pub enum Provider {
    Genius {
        parsable: bool,
    },
    #[default]
    LyricsCom,
}

impl Display for Provider {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Provider::Genius { .. } => f.write_str("Genius"),
            Provider::LyricsCom => f.write_str("Lyrics.com"),
        }
    }
}

impl From<OnlineSong> for Song {
    fn from(online_song: OnlineSong) -> Self {
        let map = if online_song.provider
            == (Provider::Genius { parsable: true })
        {
            parse_genius_lyrics(
                &online_song.lyrics.replace("\\n", "\n"),
            )
            .ok()
        } else {
            let mut map = HashMap::new();
            map.entry(VerseName::Verse { number: 1 })
                .or_insert(online_song.lyrics);
            Some(map)
        };

        Self {
            title: online_song.title,
            author: Some(online_song.author),
            verse_map: map,
            ..Default::default()
        }
    }
}

#[allow(clippy::redundant_closure_for_method_calls)]
fn parse_genius_lyrics(
    lyrics: &str,
) -> Result<HashMap<VerseName, String>> {
    let (input, chunks) =
        many1(pair(parse_verse_name, alt((take_until("["), rest))))
            .parse(lyrics)
            .map_err(|e| e.to_owned())
            .into_diagnostic()?;

    dbg!(input);
    dbg!(&chunks);

    let mut map = HashMap::new();

    for (mut name, lyric) in chunks {
        while map.contains_key(&name) {
            name = name.next();
        }

        map.entry(name).or_insert_with(|| lyric.trim().to_string());
    }

    Ok(map)
}

fn parse_verse_name(line: &str) -> IResult<&str, VerseName> {
    let (input, (name, _, num, _, _)) = delimited(
        (tag("["), space0),
        (
            take_till1(|c| c == ' ' || c == ']' || c == ':'),
            space0,
            digit0,
            alt((tag(":"), space0)),
            take_till(|c| c == ']'),
        ),
        (space0, tag("]")),
    )
    .parse(line)?;

    let num = num.parse::<usize>().unwrap_or(1);
    dbg!(&name);

    let verse_name = match name {
        "Chorus" => VerseName::Chorus { number: num },
        "Verse" => VerseName::Verse { number: num },
        "Bridge" => VerseName::Bridge { number: num },
        "Pre-Chorus" => VerseName::PreChorus { number: num },
        "Post-Chorus" => VerseName::PostChorus { number: num },
        "Outro" => VerseName::Outro { number: num },
        "Intro" => VerseName::Intro { number: num },
        "Instrumental" => VerseName::Instrumental { number: num },
        _ => VerseName::Verse { number: 99 },
    };

    Ok((input, verse_name))
}

pub async fn search_genius(
    query: String,
    auth_token: String,
) -> Result<Vec<OnlineSong>> {
    // let Some(auth_token) = option_env!("GENIUS_TOKEN") else {
    //     return Err(miette!("No Genius Token"));
    // };

    let head_value = header::HeaderValue::from_str(&auth_token)
        .into_diagnostic()?;
    let mut headers = header::HeaderMap::new();
    headers.insert(header::AUTHORIZATION, head_value);
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .into_diagnostic()?;
    let response = client
        .get(format!("https://api.genius.com/search?q={query}"))
        .send()
        .await
        .into_diagnostic()?
        .error_for_status()
        .into_diagnostic()?
        .text()
        .await
        .into_diagnostic()?;
    let json: Value =
        serde_json::from_str(&response).into_diagnostic()?;
    let hits = json
        .get("response")
        .expect("respose")
        .get("hits")
        .expect("hits")
        .as_array()
        .expect("array");
    let songs: Vec<Option<OnlineSong>> =
        cosmic::iced::futures::future::join_all(hits.iter().map(
            |hit| async {
                let result = hit.get("result").expect("result");
                let title = result
                    .get("full_title")
                    .expect("title")
                    .as_str()
                    .expect("title")
                    .to_string();
                let title = title.replace("\u{a0}", " ");
                let author = result
                    .get("artist_names")
                    .expect("artists")
                    .as_str()
                    .expect("artists")
                    .to_string();
                let link = result
                    .get("url")
                    .expect("url")
                    .as_str()
                    .expect("url")
                    .to_string();
                let song = OnlineSong {
                    lyrics: String::new(),
                    title,
                    author,
                    provider: Provider::Genius { parsable: false },
                    link,
                };

                match get_genius_lyrics(song).await {
                    Ok(song) => Some(song),
                    Err(e) => {
                        error!("Couldn't get lyrics: {e}");
                        None
                    }
                }
            },
        ))
        .await;
    Ok(songs.into_iter().filter_map(|s| s).collect())
}

pub async fn get_genius_lyrics(
    mut song: OnlineSong,
) -> Result<OnlineSong> {
    let html = reqwest::get(&song.link)
        .await
        .into_diagnostic()?
        .error_for_status()
        .into_diagnostic()?
        .text()
        .await
        .into_diagnostic()?;
    let document = scraper::Html::parse_document(&html);
    let Ok(lyrics_root_selector) = scraper::Selector::parse(
        r#"div[data-lyrics-container="true"]"#,
    ) else {
        return Err(miette!("error in finding lyrics_root"));
    };

    let lyrics = document
        .select(&lyrics_root_selector)
        .filter(|element| {
            element.attr("data-exclude-from-selection").is_none()
        })
        .filter(|element| {
            !element.value().classes().any(|class| {
                class.contains("Contrib")
                    || class.contains("LyricsHeader")
                    || class.contains("StyledLink")
            })
        })
        .flat_map(|element| {
            // dbg!(&root);
            // debug!(?element);
            let inner = element.inner_html().replace("<br>", "\n");
            // debug!(inner);
            let line_broken = scraper::Html::parse_fragment(&inner);
            line_broken
                .root_element()
                .descendent_elements()
                .filter(|element| {
                    element
                        .attr("data-exclude-from-selection")
                        .is_none()
                })
                .filter(|element| {
                    let element_name = element.value().name();
                    element_name != "div" && element_name != "path"
                })
                .filter(|element| {
                    !element.value().classes().any(|class| {
                        class.contains("Contrib")
                            || class.contains("LyricsHeader")
                            || class.contains("StyledLink")
                    })
                })
                .flat_map(|t| {
                    // let html = t.html();
                    // debug!(html);
                    t.text().collect::<Vec<&str>>()
                })
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
        })
        .collect::<String>();
    let lyrics = lyrics.find('[').map_or_else(
        || {
            lyrics.find("</div></div></div>").map_or_else(
                || lyrics.clone(),
                |position| {
                    lyrics.split_at(position + 18).1.to_string()
                },
            )
        },
        |position| lyrics.split_at(position).1.to_string(),
    );
    song.provider = Provider::Genius {
        parsable: lyrics.contains('['),
    };
    song.lyrics = lyrics;
    Ok(song)
}

pub async fn search_lyrics_com_links(
    query: impl AsRef<str> + std::fmt::Display,
) -> Result<Vec<String>> {
    let html =
        reqwest::get(format!("http://www.lyrics.com/lyrics/{query}"))
            .await
            .into_diagnostic()?
            .error_for_status()
            .into_diagnostic()?
            .text()
            .await
            .into_diagnostic()?;

    let document = scraper::Html::parse_document(&html);
    let Ok(best_matches_selector) =
        scraper::Selector::parse(".best-matches")
    else {
        return Err(miette!("error in finding matches"));
    };
    let Ok(lyric_selector) = scraper::Selector::parse("a") else {
        return Err(miette!("error in finding a links"));
    };

    Ok(document
        .select(&best_matches_selector)
        .flat_map(|best_section| best_section.select(&lyric_selector))
        .map(|a| {
            a.value().attr("href").unwrap_or("").trim().to_string()
        })
        .filter(|a| a.contains("/lyric/"))
        .dedup()
        .map(|link| {
            link.strip_prefix("/lyric/")
                .unwrap_or_else(|| &link)
                .to_string()
        })
        .collect())
}

// leaving this lint unfixed because I don't know if we will need this
// id value or not in the future and I'd like to keep the code understanding
// of what this variable might be.
#[allow(clippy::no_effect_underscore_binding)]
pub async fn lyrics_com_link_to_song(
    links: Vec<impl AsRef<str> + std::fmt::Display>,
) -> Result<Vec<OnlineSong>> {
    let mut songs = vec![];
    for link in links {
        let parts = link
            .as_ref()
            .split('/')
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>();
        let link = format!("https://www.lyrics.com/lyric/{link}");
        let _id = &parts[0];
        let author = &parts[1].replace('+', " ");
        let title = &parts[2].replace('+', " ");

        let html = reqwest::get(&link)
            .await
            .into_diagnostic()?
            .error_for_status()
            .into_diagnostic()?
            .text()
            .await
            .into_diagnostic()?;

        let document = scraper::Html::parse_document(&html);
        let Ok(lyric_selector) =
            scraper::Selector::parse(".lyric-body")
        else {
            return Err(miette!("error in finding lyric-body",));
        };

        let lyrics = document
            .select(&lyric_selector)
            .map(|a| a.text().collect::<String>())
            .dedup()
            .next();

        if let Some(lyrics) = lyrics {
            let song = OnlineSong {
                lyrics,
                title: title.clone(),
                author: author.clone(),
                provider: Provider::LyricsCom,
                link,
            };

            songs.push(song);
        }
    }
    Ok(songs)
}

#[cfg(test)]
mod test {
    use crate::core::songs::Song;

    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn genius() -> Result<(), String> {
        let song = OnlineSong {
            lyrics: String::new(),
            title: "Death Was Arrested by North Point Worship (Ft. Seth Condrey)".to_string(),
            author: "North Point Worship (Ft. Seth Condrey)".to_string(),
            provider: Provider::Genius { parsable: false },
            link: "https://genius.com/North-point-worship-death-was-arrested-lyrics".to_string(),
        };
        let hits = search_genius(
            "Death was arrested".to_string(),
            env!("GENIUS_TOKEN").to_string(),
        )
        .await
        .map_err(|e| e.to_string())?;

        assert!(
            hits.contains(&song),
            "There was no song that matched on Genius"
        );

        let titles: Vec<String> =
            hits.iter().map(|song| song.title.clone()).collect();
        dbg!(titles);
        for hit in hits {
            let new_song = get_genius_lyrics(hit)
                .await
                .map_err(|e| e.to_string())?;
            dbg!(&new_song);
            dbg!(&new_song.provider);
            if new_song.lyrics.starts_with("[Verse 1]") {
                assert!(new_song.lyrics.contains("[Verse 2]"));
                if !new_song.lyrics.contains("[Chorus]") {
                    assert!(new_song.lyrics.contains("[Chorus 1]"));
                }
            } else {
                assert!(new_song.lyrics.len() > 10);
            }
            let mapped_song = Song::from(new_song);
            dbg!(&mapped_song);
            if let Some(map) = mapped_song.verse_map.as_ref() {
                assert!(map.len() > 3);
                assert!(
                    map.keys()
                        .contains(&VerseName::Verse { number: 1 })
                        && map.keys().contains(&VerseName::Verse {
                            number: 2
                        })
                        && map.keys().contains(&VerseName::Chorus {
                            number: 1
                        })
                );
            } else {
                assert!(
                    !mapped_song
                        .lyrics
                        .is_some_and(|lyrics| lyrics.contains("["))
                )
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn search_to_song() -> Result<(), String> {
        let song = OnlineSong {
            lyrics: "Alone in my sorrow and dead in my sin\nLost without hope with no place to begin\nYour love Made a way to let mercy come in\nWhen death was arrested and my life began\n\nAsh was redeemed only beauty remains\nMy orphan heart was given a name\nMy mourning grew quiet my feet rose to dance\nWhen death was arrested and my life began\n\nOh, Your grace so free\nWashes over me\nYou have made me new\nNow life begins with You\nIt's your endless love\nPouring down on us\nYou have made us new\nNow life begins with You\n\nReleased from my chains I'm a prisoner no more\nMy shame was a ransom He faithfully bore\nHe cancelled my debt and He called me His friend\nWhen death was arrested and my life began\n\nOh, Your grace so free\nWashes over me\nYou have made me new\nNow life begins with You\nIt's your endless love\nPouring down on us\nYou have made us new\nNow life begins with You\n\nOur savior displayed on a criminal's cross\nDarkness rejoiced as though heaven had lost\nBut then Jesus arose with our freedom in hand\nThat's when death was arrested and my life began\n\nOh, Your grace so free\nWashes over me\nYou have made me new\nNow life begins with You\nIt's your endless love\nPouring down on us\nYou have made us new\nNow life begins with You\n\nOh, we're free, free\nForever we're free\nCome join the song\nOf all the redeemed\nYes, we're free free\nForever amen\nWhen death was arrested and my life began\n\nOh, we're free, free\nForever we're free\nCome join the song\nOf all the redeemed\nYes, we're free free\nForever amen\nWhen death was arrested and my life began\n\nWhen death was arrested and my life began\nWhen death was arrested and my life began".to_string(),
            title: "Death Was Arrested".to_string(),
            author: "North Point InsideOut".to_string(),
            provider: Provider::LyricsCom,
            link: "https://www.lyrics.com/lyric/35090938/North+Point+InsideOut/Death+Was+Arrested".to_string(),
        };
        let links = search_lyrics_com_links("Death was arrested")
            .await
            .map_err(|e| format!("{e}"))?;
        let songs = lyrics_com_link_to_song(links)
            .await
            .map_err(|e| format!("{e}"))?;
        if let Some(first) = songs.iter().find_or_first(|song| {
            song.author == "North Point InsideOut"
        }) {
            assert_eq!(&song, first);
            online_song_to_song(song)?;
        }
        Ok(())
    }

    fn online_song_to_song(song: OnlineSong) -> Result<(), String> {
        let song = Song::from(song);
        if let Some(verse_map) = song.verse_map.as_ref() {
            if verse_map.len() < 2 {
                return Err(format!(
                    "VerseMap wasn't built right likely: {song:?}",
                ));
            }
        } else {
            return Err(String::from(
                "There is no VerseMap in this song",
            ));
        }
        Ok(())
    }

    #[tokio::test]
    async fn online_search() {
        let search =
            search_lyrics_com_links("Death was arrested").await;
        match search {
            Ok(songs) => {
                assert_eq!(
                    songs,
                    vec![
                        "33755723/Various+Artists/Death+Was+Arrested",
                        "35090938/North+Point+InsideOut/Death+Was+Arrested"
                    ]
                );
            }
            Err(e) => panic!("{e}"),
        }
    }

    #[test]
    #[allow(clippy::redundant_closure_for_method_calls)]
    fn test_parse_verse_name() -> Result<()> {
        let names = [
            "[ Chorus ]",
            "[Verse 1]",
            "[Pre-Chorus]",
            "[ Post-Chorus ]",
            "[ Post-Chorus 3]",
            "[Verse 2]",
            "[Verse 3]",
            "[Verse 4:]",
            "[Verse 5: Coffee]",
            "[Chorus 1]",
            "[ Chorus 2 ]",
        ];
        for name in names {
            let (_input, parsed) = parse_verse_name
                .parse(name)
                .map_err(|e| e.to_owned())
                .into_diagnostic()?;
            dbg!(parsed);
        }
        Ok(())
    }

    #[test]
    fn test_parse_song() -> Result<()> {
        let song = r#"[Verse 1]
Glory, glory
I've been singing
Since I laid my burden down
Glory, glory
I've been singing
Since I laid my burden down

[Chorus]
I'm singing, "Hallelujah"
God is able, hallelujah
God is faithful, hallelujah
Lord, I'm gonna sing

[Verse 2]
I feel better
So much better
Since I laid my burden down
Yeah, I feel better
So much better
Since I laid, O Lord, I laid my burden down

[Chorus]
I'm singing, "Hallelujah"
God is able, hallelujah
God is faithful, hallelujah
Lord, I'm gonna sing
I'm singing, "Hallelujah"
God is able, hallelujah
God is faithful, hallelujah
Lord, I'm gonna sing

[Bridge]
As long as I'm alive there's gonna be praising
As long as I'm alive there's gonna be shouting
One thing that I know, oh, deep down in my soul
As long as I'm alive, I'm gonna sing

[Chorus]
I'm singing, "Hallelujah" (Hallelujah)
God is able, hallelujah (Hallelujah)
God is faithful, hallelujah
Lord, I'm gonna sing (Come on now, sing it)
Oh I'm singing, "Hallelujah" (Hallelujah)
God is able, hallelujah (Hallelujah)
God is faithful, hallelujah (God is so good)
Lord, I'm gonna sing (Sing it, Dave)

[Outro]
I'm gonna sing
Aw man, that was good"#;
        let new_song = r#"[Verse 1]\nAlone in my sorrow and dead in my sin\nLost without hope with no place to begin\nYour love made a way to let mercy come in\nWhen death was arrested and my life began\nAsh was redeemed, only beauty remains\nMy orphan heart was given a name\nMy mourning grew quiet, my feet rose to dance\nWhen death was arrested and my life began\n\n[Chorus]\nOh, Your grace so free, washes over me\nYou have made me new, now life begins with You\nIt's Your endless love, pouring down on us\nYou have made us new, now life begins with You\n\n[Verse 2]\nReleased from my chains, I'm a prisoner no more\nMy shame was a ransom He faithfully bore\nHe cancelled my debt and He called me His friend\nWhen death was arrested and my life began\n\n[Chorus]\nOh, Your grace so free, washes over me\nYou have made me new, now life begins with You\nIt's Your endless love, pouring down on us\nYou have made us new, now life begins with You\n[Verse 3]\nOur Savior displayed on a criminal's cross\nDarkness rejoiced as though heaven had lost\nBut then Jesus arose with our freedom in hand\nThat's when death was arrested and my life began\n\n[Chorus]\nOh, Your grace so free, washes over me\nYou have made me new, now life begins with You\nIt's Your endless love, pouring down on us\nYou have made us new, now life begins with You\n\n[Outro]\nOh, we're free, free, forever we're free\nCome join the song of all the redeemed\nYes, we're free, free, forever amen\nWhen death was arrested and my life began\nOh, we're free, free, forever we're free\nCome join the song of all the redeemed\nYes, we're free, free, forever amen\nWhen death was arrested and my life began\nWhen death was arrested and my life began\nWhen death was arrested and my life began"#.replace("\\n", "\n");
        let map = parse_genius_lyrics(song)?;
        let new_map = parse_genius_lyrics(&new_song)?;
        dbg!(map);
        dbg!(new_map);
        Ok(())
    }
}
