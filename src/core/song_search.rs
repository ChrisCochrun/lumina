use itertools::Itertools;
use miette::{IntoDiagnostic, Result};

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Ord, Eq)]
pub struct OnlineSong {
    lyrics: String,
    title: String,
    author: String,
    site: String,
    link: String,
}

pub async fn search_online_song_links(
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
    let best_matches_selector =
        scraper::Selector::parse(".best-matches").unwrap();
    let lyric_selector = scraper::Selector::parse("a").unwrap();

    Ok(document
        .select(&best_matches_selector)
        .filter_map(|best_section| {
            Some(best_section.select(&lyric_selector))
        })
        .flatten()
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

pub async fn link_to_online_song(
    links: Vec<impl AsRef<str> + std::fmt::Display>,
) -> Result<Vec<OnlineSong>> {
    let mut songs = vec![];
    for link in links {
        let parts = link
            .to_string()
            .split('/')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let link = format!("https://www.lyrics.com/lyric/{link}");
        dbg!(&link);
        let _id = &parts[0];
        let author = &parts[1].replace("+", " ");
        let title = &parts[2].replace("+", " ");

        let html = reqwest::get(&link)
            .await
            .into_diagnostic()?
            .error_for_status()
            .into_diagnostic()?
            .text()
            .await
            .into_diagnostic()?;

        let document = scraper::Html::parse_document(&html);
        let lyric_selector =
            scraper::Selector::parse(".lyric-body").unwrap();

        let lyrics = document
            .select(&lyric_selector)
            .map(|a| {
                dbg!(&a);
                a.text().collect::<String>()
            })
            .dedup()
            .next();

        dbg!(&lyrics);
        if let Some(lyrics) = lyrics {
            let song = OnlineSong {
                lyrics,
                title: title.to_string(),
                author: author.to_string(),
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
    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_search_to_song() {
        let song = OnlineSong {
            lyrics: "Bla".to_string(),
            title: "Bla".to_string(),
            author: "Bla".to_string(),
            site: "Bla".to_string(),
            link: "Bla".to_string(),
        };
        let vec = vec![song];
        let search =
            search_online_song_links("Death was arrested").await;
        match search {
            Ok(links) => {
                let songs = link_to_online_song(links).await;
                match songs {
                    Ok(songs) => {
                        assert_eq!(songs, vec);
                    }
                    Err(e) => assert!(false, "{}", e),
                }
            }
            Err(e) => assert!(false, "{}", e),
        }
    }

    #[tokio::test]
    async fn test_online_search() {
        let search =
            search_online_song_links("Death was arrested").await;
        match search {
            Ok(songs) => {
                assert_eq!(songs, vec!["hello"]);
                for song in songs {
                    assert_eq!(
                        "/lyric/33755723/Various+Artists/Death+Was+Arrested",
                        song
                    )
                }
            }
            Err(e) => assert!(false, "{}", e),
        }
    }
}
