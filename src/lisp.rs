use std::{fs::read_to_string, path::PathBuf};

use crisp::types::{Symbol, Value};

use crate::core::service_items::ServiceItem;

pub fn parse_lisp(value: Value) -> Vec<ServiceItem> {
    match &value {
        Value::List(vec) => match &vec[0] {
            Value::Symbol(Symbol(s))
                if s == "slide" || s == "song" =>
            {
                let item = ServiceItem::from(value.clone());
                vec![item]
            }
            Value::Symbol(Symbol(s)) if s == "load" => {
                let Ok(path) = PathBuf::from(String::from(&vec[1]))
                    .canonicalize()
                else {
                    return vec![ServiceItem::default()];
                };
                let lisp = read_to_string(path).expect("oops");
                let lisp_value = crisp::reader::read(&lisp);
                match lisp_value {
                    Value::List(value) => value
                        .into_iter()
                        .flat_map(parse_lisp)
                        .collect(),
                    _ => panic!("Should not be"),
                }
            }
            _ => todo!(),
        },
        _ => todo!(),
    }
}

#[cfg(test)]
mod test {
    use std::{fs::read_to_string, path::PathBuf};

    use crate::{
        core::{
            images::Image, kinds::ServiceItemKind, songs::Song,
            videos::Video,
        },
        Background, SlideBuilder, TextAlignment,
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parsing_lisp() {
        let lisp =
            read_to_string("./test_slides.lisp").expect("oops");
        let lisp_value = crisp::reader::read(&lisp);
        let test_vec = vec![service_item_1(), service_item_2()];
        match lisp_value {
            Value::List(value) => {
                let mut item_vec = vec![];
                for value in value {
                    let mut vec = parse_lisp(value);
                    item_vec.append(&mut vec);
                }
                assert_eq!(item_vec, test_vec)
            }
            _ => panic!("this should be a lisp"),
        }
    }

    #[test]
    fn test_parsing_lisp_presentation() {
        let lisp = read_to_string("./testypres.lisp").expect("oops");
        let lisp_value = crisp::reader::read(&lisp);
        let test_vec = vec![
            service_item_1(),
            service_item_2(),
            service_item_3(),
        ];
        match lisp_value {
            Value::List(value) => {
                let mut item_vec = vec![];
                for value in value {
                    let mut vec = parse_lisp(value);
                    item_vec.append(&mut vec);
                }
                let item_1 = &item_vec[0];
                let item_2 = &item_vec[1];
                let item_3 = &item_vec[2];
                assert_eq!(item_1, &test_vec[0]);
                assert_eq!(item_2, &test_vec[1]);
                assert_eq!(item_3, &test_vec[2]);

                assert_eq!(item_vec, test_vec);
            }
            _ => panic!("this should be a lisp"),
        }
    }

    fn service_item_1() -> ServiceItem {
        ServiceItem {
            title: "frodo.jpg".to_string(),
            kind: ServiceItemKind::Image(Image {
                title: "frodo.jpg".to_string(),
                path: PathBuf::from("~/pics/frodo.jpg"),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn service_item_2() -> ServiceItem {
        ServiceItem {
            title: "camprules2024.mp4".to_string(),
            kind: ServiceItemKind::Video(Video {
                title: "camprules2024.mp4".to_string(),
                path: PathBuf::from("~/vids/test/camprules2024.mp4"),
                start_time: None,
                end_time: None,
                looping: false,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn service_item_3() -> ServiceItem {
        ServiceItem {
            title: "Death Was Arrested".to_string(),
            kind: ServiceItemKind::Song(test_song()),
            database_id: 7,
            ..Default::default()
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
