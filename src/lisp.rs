use std::{fs::read_to_string, path::PathBuf};

use crisp::types::{Symbol, Value};
use tracing::error;

use crate::{core::songs::lisp_to_song, Slide};

pub fn parse_lisp(value: Value) -> Vec<Slide> {
    match &value {
        Value::List(vec) => match &vec[0] {
            Value::Symbol(Symbol(s)) if s == "slide" => {
                let slide = Slide::from(value.clone());
                vec![slide]
            }
            Value::Symbol(Symbol(s)) if s == "song" => {
                let song = lisp_to_song(vec.clone());
                match Slide::song_slides(&song) {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Couldn't load song: {e}");
                        vec![Slide::default()]
                    }
                }
            }
            Value::Symbol(Symbol(s)) if s == "load" => {
                let Ok(path) = PathBuf::from(String::from(&vec[1]))
                    .canonicalize()
                else {
                    return vec![Slide::default()];
                };
                let lisp = read_to_string(path).expect("oops");
                let lisp_value = crisp::reader::read(&lisp);
                match lisp_value {
                    Value::List(value) => value
                        .into_iter()
                        .flat_map(|v| parse_lisp(v))
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

    use crate::{Background, SlideBuilder, TextAlignment};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parsing_lisp() {
        let lisp =
            read_to_string("./test_slides.lisp").expect("oops");
        let lisp_value = crisp::reader::read(&lisp);
        let test_vec = vec![test_slide(), test_second_slide()];
        match lisp_value {
            Value::List(value) => {
                let mut slide_vec = vec![];
                for value in value {
                    let mut vec = parse_lisp(value);
                    slide_vec.append(&mut vec);
                }
                assert_eq!(slide_vec, test_vec)
            }
            _ => panic!("this should be a lisp"),
        }
    }

    #[test]
    fn test_parsing_lisp_presentation() {
        let lisp =
            read_to_string("./test_presentation.lisp").expect("oops");
        let lisp_value = crisp::reader::read(&lisp);
        let test_vec =
            vec![test_slide(), test_second_slide(), song_slide()];
        match lisp_value {
            Value::List(value) => {
                let mut slide_vec = vec![];
                for value in value {
                    let mut vec = parse_lisp(value);
                    slide_vec.append(&mut vec);
                }
                let first_lisp_slide = &slide_vec[0];
                let second_lisp_slide = &slide_vec[1];
                let third_lisp_slide = &slide_vec[2];
                assert_eq!(first_lisp_slide, &test_vec[0]);
                assert_eq!(second_lisp_slide, &test_vec[1]);
                assert_eq!(third_lisp_slide, &test_vec[2]);

                assert_eq!(slide_vec, test_vec);
            }
            _ => panic!("this should be a lisp"),
        }
    }

    fn test_slide() -> Slide {
        SlideBuilder::new()
            .text("This is frodo")
            .background(
                Background::try_from("~/pics/frodo.jpg").unwrap(),
            )
            .font("Quicksand")
            .font_size(70)
            .text_alignment(TextAlignment::MiddleCenter)
            .video_loop(false)
            .video_start_time(0.0)
            .video_end_time(0.0)
            .build()
            .unwrap()
    }

    fn test_second_slide() -> Slide {
        SlideBuilder::new()
            .text("")
            .background(
                Background::try_from("~/vids/test/camprules2024.mp4")
                    .unwrap(),
            )
            .font("Quicksand")
            .font_size(0)
            .text_alignment(TextAlignment::MiddleCenter)
            .video_loop(false)
            .video_start_time(0.0)
            .video_end_time(0.0)
            .build()
            .unwrap()
    }

    fn song_slide() -> Slide {
        SlideBuilder::new()
            .text("Death Was Arrested\nNorth Point Worship")
            .background(
                Background::try_from("~/nc/tfc/openlp/CMG - Bright Mountains 01.jpg")
                    .unwrap(),
            )
            .font("Quicksand Bold")
            .font_size(60)
            .text_alignment(TextAlignment::MiddleCenter)
            .audio(PathBuf::from("file:///home/chris/music/North Point InsideOut/Nothing Ordinary, Pt. 1 (Live)/05 Death Was Arrested (feat. Seth Condrey).mp3"))
            .video_loop(true)
            .video_start_time(0.0)
            .video_end_time(0.0)
            .build()
            .unwrap()
    }
}
