use reqwest::Error;
use serde_derive::Deserialize;
use serde_json::Value;
use tracing::debug;

#[derive(PartialEq, Debug, Eq, Deserialize)]
pub struct Song {
    title: String,
    lyrics: String,
    id: u32,
    artist_names: String,
}

#[derive(Debug)]
pub struct SongBuilder {
    title: Option<String>,
    lyrics: Option<String>,
    id: Option<u32>,
    artist_names: Option<String>,
}

impl Song {
    fn builder() -> SongBuilder {
        SongBuilder {
            title: None,
            lyrics: None,
            id: None,
            artist_names: None,
        }
    }
}

impl SongBuilder {
    fn title(mut self, s: &str) -> SongBuilder {
        self.title = Some(s.to_owned());
        self
    }

    fn artist(mut self, s: &str) -> SongBuilder {
        self.artist_names = Some(s.to_owned());
        self
    }

    fn id(mut self, id: u32) -> SongBuilder {
        self.id = Some(id);
        self
    }

    fn build(self) -> Song {
        Song {
            title: self.title.unwrap_or_default(),
            lyrics: self.lyrics.unwrap_or_default(),
            id: self.id.unwrap_or_default(),
            artist_names: self.artist_names.unwrap_or_default(),
        }
    }
}

pub fn search_song(s: &str) -> Result<Vec<Song>, Error> {
    let url = String::from("https://api.genius.com/search?");
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let req = reqwest::Client::new()
            .get(url)
            .query(&[("access_token", "R0Y0ZW50Il9LSh5su3LKfdyfmQRx41NpVvLFJ0VxMo-hQ_4H1OVg_IE0Q-UUoFQx"), ("q", s)])
            .send().await?
            .text().await?;

        let json: Value = serde_json::from_str(req.as_str()).unwrap();
        let mut songs: Vec<Song> = vec![];
        let collection = json["response"]["hits"].as_array();
        if let Some(col) = collection {
            for result in col {
                let result = &result["result"];
                let song = Song::builder()
                    .title(result["title"].as_str().unwrap_or_default())
                    .artist(result["artist_names"].as_str().unwrap_or_default())
                    .id(result["id"].as_u64().unwrap_or_default() as u32).build();
                // debug!(title = ?result["title"], artist = ?result["artist_names"]);
                songs.push(song);
            }
        }
        debug!(?songs);
        Ok(songs)
    })
}

#[cfg(test)]
mod tests {
    use tracing_subscriber::EnvFilter;

    use super::*;

    #[test]
    fn test_search() {
        // Uncomment to see debug info
        tracing_subscriber::FmtSubscriber::builder()
            .pretty()
            .with_line_number(true)
            .with_level(true)
            .with_target(true)
            .with_env_filter(EnvFilter::from_default_env())
            .init();

        let song = "Perfect";
        let res =
            search_song(song).unwrap().into_iter().next().unwrap();
        let song = Song {
            title: String::from("Perfect"),
            lyrics: String::from(""),
            id: 2953761,
            artist_names: String::from("Ed Sheeran"),
        };
        assert_eq!(song, res);
    }
}
