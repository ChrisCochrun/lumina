use std::collections::HashMap;

use crate::core::settings;
use crate::core::songs::{Song, VerseName};
use itertools::Itertools;
use miette::{IntoDiagnostic, Result, miette};
use nom::branch::alt;
use nom::bytes::{tag, take_till, take_till1, take_until};
use nom::character::complete::{digit0, newline, space0};
use nom::multi::{many0, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated};
use nom::{IResult, Parser};
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    pub site: String,
    pub link: String,
}

impl From<OnlineSong> for Song {
    fn from(value: OnlineSong) -> Self {
        let mut song = Self {
            verse_map: Some(HashMap::new()),
            ..Default::default()
        };

        for line in value.lyrics.lines() {
            let next_verse = match line {
                "[Chorus]" => VerseName::Chorus { number: 1 },
                _ => song.get_next_verse_name(),
            };
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

fn parse_genius_lyrics(
    lyrics: &str,
) -> Result<HashMap<VerseName, String>> {
    let (input, chunks) =
        separated_list1(pair(newline, newline), many0(ident))
            .parse(lyrics)
            .map_err(|e| e.to_owned())
            .into_diagnostic()?;

    dbg!(input);
    dbg!(&chunks);

    let mut map = HashMap::new();

    for chunk in chunks {
        dbg!(&chunk);
        let chunk = chunk.join("\n");
        let (_, (mut name, lyric)) = parse_verse
            .parse(&chunk)
            .map_err(|e| e.to_owned())
            .into_diagnostic()?;
        while map.contains_key(&name) {
            name = name.next();
        }

        map.entry(name).or_insert(lyric);
    }

    Ok(map)
}

fn ident(input: &str) -> IResult<&str, &str> {
    preceded(alt((tag("\n\n"), space0)), any).parse(input)
}

fn any(input: &str) -> IResult<&str, &str> {
    take_until("\n\n").parse(input)
}

fn block(input: &str) -> IResult<&str, &str> {
    terminated(ident, pair(newline, newline)).parse(input)
}

fn parse_verse(chunk: &str) -> IResult<&str, (VerseName, String)> {
    let (input, verse_name) = parse_verse_name.parse(chunk)?;
    let lyrics = input.trim().to_string();
    Ok((input, (verse_name, lyrics)))
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

fn parse_verse_lyrics(lyrics: &str) -> IResult<&str, String> {
    todo!()
}

pub async fn search_genius_links(
    query: impl AsRef<str> + std::fmt::Display,
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
    Ok(hits
        .iter()
        .map(|hit| {
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
            OnlineSong {
                lyrics: String::new(),
                title,
                author,
                site: String::from("https://genius.com"),
                link,
            }
        })
        .collect())
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
        .map(|root| {
            // dbg!(&root);
            root.inner_html()
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
    let lyrics = lyrics.replace("<br>", "\n");
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
                site: "https://www.lyrics.com".into(),
                link,
            };

            songs.push(song);
        }
    }
    Ok(songs)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::core::songs::Song;

    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn genius() -> Result<(), String> {
        let song = OnlineSong {
            lyrics: String::new(),
            title: "Death Was Arrested by North Point Worship (Ft. Seth Condrey)".to_string(),
            author: "North Point Worship (Ft. Seth Condrey)".to_string(),
            site: "https://genius.com".to_string(),
            link: "https://genius.com/North-point-worship-death-was-arrested-lyrics".to_string(),
        };
        let hits = search_genius_links(
            "Death was arrested",
            "test".to_string(),
        )
        .await
        .map_err(|e| e.to_string())?;

        assert!(
            hits.iter().find(|hit| **hit == song).is_some(),
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
            if !new_song.lyrics.starts_with("[Verse 1]") {
                assert!(new_song.lyrics.len() > 10);
            } else {
                assert!(new_song.lyrics.contains("[Verse 2]"));
                if !new_song.lyrics.contains("[Chorus]") {
                    assert!(new_song.lyrics.contains("[Chorus 1]"))
                }
            }
            let mapped_song = Song::from(new_song);
            dbg!(&mapped_song);
            assert!(
                mapped_song
                    .verse_map
                    .as_ref()
                    .is_some_and(|map| map.len() > 3)
            );
            assert!(Some(HashMap::new()) == mapped_song.verse_map)
        }

        Ok(())
    }

    #[tokio::test]
    async fn search_to_song() -> Result<(), String> {
        let song = OnlineSong {
            lyrics: "Alone in my sorrow and dead in my sin\nLost without hope with no place to begin\nYour love Made a way to let mercy come in\nWhen death was arrested and my life began\n\nAsh was redeemed only beauty remains\nMy orphan heart was given a name\nMy mourning grew quiet my feet rose to dance\nWhen death was arrested and my life began\n\nOh, Your grace so free\nWashes over me\nYou have made me new\nNow life begins with You\nIt's your endless love\nPouring down on us\nYou have made us new\nNow life begins with You\n\nReleased from my chains I'm a prisoner no more\nMy shame was a ransom He faithfully bore\nHe cancelled my debt and He called me His friend\nWhen death was arrested and my life began\n\nOh, Your grace so free\nWashes over me\nYou have made me new\nNow life begins with You\nIt's your endless love\nPouring down on us\nYou have made us new\nNow life begins with You\n\nOur savior displayed on a criminal's cross\nDarkness rejoiced as though heaven had lost\nBut then Jesus arose with our freedom in hand\nThat's when death was arrested and my life began\n\nOh, Your grace so free\nWashes over me\nYou have made me new\nNow life begins with You\nIt's your endless love\nPouring down on us\nYou have made us new\nNow life begins with You\n\nOh, we're free, free\nForever we're free\nCome join the song\nOf all the redeemed\nYes, we're free free\nForever amen\nWhen death was arrested and my life began\n\nOh, we're free, free\nForever we're free\nCome join the song\nOf all the redeemed\nYes, we're free free\nForever amen\nWhen death was arrested and my life began\n\nWhen death was arrested and my life began\nWhen death was arrested and my life began".to_string(),
            title: "Death Was Arrested".to_string(),
            author: "North Point InsideOut".to_string(),
            site: "https://www.lyrics.com".to_string(),
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
            online_song_to_song(song)?
        }
        Ok(())
    }

    fn online_song_to_song(song: OnlineSong) -> Result<(), String> {
        let song = Song::from(song);
        if let Some(verse_map) = song.verse_map.as_ref() {
            if verse_map.len() < 2 {
                return Err(format!(
                    "VerseMap wasn't built right likely: {:?}",
                    song
                ));
            }
        } else {
            return Err(String::from(
                "There is no VerseMap in this song",
            ));
        };
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
            Err(e) => assert!(false, "{}", e),
        }
    }

    #[test]
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
            let (input, parsed) = parse_verse_name
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
        let map = parse_genius_lyrics(song)?;
        dbg!(map);
        assert!(false);
        Ok(())
    }

    #[test]
    fn test_block_parsing() -> Result<()> {
        let chorus = r#"[Chorus]
I'm singing, "Hallelujah" (Hallelujah)
God is able, hallelujah (Hallelujah)
God is faithful, hallelujah
Lord, I'm gonna sing (Come on now, sing it)
Oh I'm singing, "Hallelujah" (Hallelujah)
God is able, hallelujah (Hallelujah)
God is faithful, hallelujah (God is so good)
Lord, I'm gonna sing (Sing it, Dave)

"#;
        let thing = block.parse(chorus).into_diagnostic()?;
        Ok(())
    }
}
