use diesel::prelude::*;
use std::collections::HashMap;
use tracing::{debug, debug_span, error, info, instrument};

#[derive(Default, Clone, Debug, Queryable)]
pub struct Song {
    pub id: i32,
    pub title: String,
    pub lyrics: Option<String>,
    pub author: Option<String>,
    pub ccli: Option<String>,
    pub audio: Option<String>,
    pub verse_order: Option<String>,
    pub background: Option<String>,
    pub background_type: Option<String>,
    pub horizontal_text_alignment: Option<String>,
    pub vertical_text_alignment: Option<String>,
    pub font: Option<String>,
    pub font_size: Option<i32>,
}

impl Song {
    pub fn get_lyric_list(&self) -> Vec<String> {
        let mut lyric_list: Vec<String> = vec![];
        if self.lyrics.clone().unwrap_or_default().is_empty() {
            return lyric_list;
        }
        let raw_lyrics = self.lyrics.clone().unwrap_or_default();
        println!("raw-lyrics: {:?}", raw_lyrics);
        let verse_order =
            self.verse_order.clone().unwrap_or_default();
        let vorder: Vec<&str> = verse_order.split(' ').collect();
        let keywords = vec![
            "Verse 1", "Verse 2", "Verse 3", "Verse 4", "Verse 5",
            "Verse 6", "Verse 7", "Verse 8", "Chorus 1", "Chorus 2",
            "Chorus 3", "Chorus 4", "Bridge 1", "Bridge 2",
            "Bridge 3", "Bridge 4", "Intro 1", "Intro 2", "Ending 1",
            "Ending 2", "Other 1", "Other 2", "Other 3", "Other 4",
        ];
        let mut first_item = true;

        let mut lyric_map = HashMap::new();
        let mut verse_title = String::from("");
        let mut lyric = String::from("");
        for (i, line) in raw_lyrics.split("\n").enumerate() {
            if keywords.contains(&line) {
                if i != 0 {
                    println!("{verse_title}");
                    println!("{lyric}");
                    lyric_map.insert(verse_title, lyric);
                    lyric = String::from("");
                    verse_title = line.to_string();
                    // println!("{line}");
                    // println!("\n");
                } else {
                    verse_title = line.to_string();
                    // println!("{line}");
                    // println!("\n");
                }
            } else {
                lyric.push_str(line);
                lyric.push_str("\n");
            }
        }
        lyric_map.insert(verse_title, lyric);
        println!("da-map: {:?}", lyric_map);

        for mut verse in vorder {
            let mut verse_name = "";
            debug!(verse = verse);
            for word in keywords.clone() {
                let end_verse = verse.get(1..2).unwrap_or_default();
                let beg_verse = verse.get(0..1).unwrap_or_default();
                debug!(
                    verse,
                    beginning = beg_verse,
                    end = end_verse,
                    word
                );
                if word.starts_with(beg_verse)
                    && word.ends_with(end_verse)
                {
                    verse_name = word;
                    debug!(title = verse_name);
                    continue;
                }
            }
            if let Some(lyric) = lyric_map.get(verse_name) {
                if lyric.contains("\n\n") {
                    let split_lyrics: Vec<&str> =
                        lyric.split("\n\n").collect();
                    for lyric in split_lyrics {
                        if lyric == "" {
                            continue;
                        }
                        lyric_list.push(lyric.to_owned());
                    }
                    continue;
                }
                lyric_list.push(lyric.clone());
            } else {
                println!("NOT WORKING!");
            };
        }
        for lyric in lyric_list.iter() {
            debug!(lyric = lyric);
        }
        lyric_list
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cxx_qt_lib::QStringList; // Replace 'some_module' with the actual module where QModelIndex is defined.

//     #[test]
//     fn test_get_lyric_list() {
//         // Create a test instance of your struct (you might need to adjust this based on your actual struct).
//         let mut song_model = SongModel {
//             highest_id: 0,
//             songs: Vec::<Song>::new(),
//         };

//         let cpp =
//             Pin::<&mut song_model::SongModelQt> { _private: todo!() };

//         // this sets up the songmodel with the database
//         song_model.setup_wrapper(cpp);

//         // Call the get_lyric_list function with specific inputs.
//         let index = 0; // Replace with your desired test index.

//         let result = song_model.get_lyric_list_wrapper(cpp, index);

//         // Define your expected result here. For simplicity, let's assume an empty QStringList.
//         let expected_result = QStringList::default();

//         // Compare the actual result with the expected result using an assertion.
//         assert_eq!(result, expected_result);
//     }
// }
