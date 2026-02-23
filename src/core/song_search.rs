use itertools::Itertools;
use miette::{IntoDiagnostic, Result, miette};

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Ord, Eq)]
pub struct OnlineSong {
    pub lyrics: String,
    pub title: String,
    pub author: String,
    pub site: String,
    pub link: String,
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
pub async fn link_to_online_song(
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
    use crate::core::songs::Song;

    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_search_to_song() -> Result<(), String> {
        let song = OnlineSong {
            lyrics: "Alone in my sorrow and dead in my sin\nLost without hope with no place to begin\nYour love Made a way to let mercy come in\nWhen death was arrested and my life began\n\nAsh was redeemed only beauty remains\nMy orphan heart was given a name\nMy mourning grew quiet my feet rose to dance\nWhen death was arrested and my life began\n\nOh, Your grace so free\nWashes over me\nYou have made me new\nNow life begins with You\nIt's your endless love\nPouring down on us\nYou have made us new\nNow life begins with You\n\nReleased from my chains I'm a prisoner no more\nMy shame was a ransom He faithfully bore\nHe cancelled my debt and He called me His friend\nWhen death was arrested and my life began\n\nOh, Your grace so free\nWashes over me\nYou have made me new\nNow life begins with You\nIt's your endless love\nPouring down on us\nYou have made us new\nNow life begins with You\n\nOur savior displayed on a criminal's cross\nDarkness rejoiced as though heaven had lost\nBut then Jesus arose with our freedom in hand\nThat's when death was arrested and my life began\n\nOh, Your grace so free\nWashes over me\nYou have made me new\nNow life begins with You\nIt's your endless love\nPouring down on us\nYou have made us new\nNow life begins with You\n\nOh, we're free, free\nForever we're free\nCome join the song\nOf all the redeemed\nYes, we're free free\nForever amen\nWhen death was arrested and my life began\n\nOh, we're free, free\nForever we're free\nCome join the song\nOf all the redeemed\nYes, we're free free\nForever amen\nWhen death was arrested and my life began\n\nWhen death was arrested and my life began\nWhen death was arrested and my life began".to_string(),
            title: "Death Was Arrested".to_string(),
            author: "North Point InsideOut".to_string(),
            site: "https://www.lyrics.com".to_string(),
            link: "https://www.lyrics.com/lyric/35090938/North+Point+InsideOut/Death+Was+Arrested".to_string(),
        };
        let links = search_online_song_links("Death was arrested")
            .await
            .map_err(|e| format!("{e}"))?;
        let songs = link_to_online_song(links)
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
    async fn test_online_search() {
        let search =
            search_online_song_links("Death was arrested").await;
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
}
