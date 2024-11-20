use crisp::types::{Symbol, Value};

use crate::Slide;

pub fn parse_lisp(value: Value) -> Vec<Slide> {
    match &value {
        Value::List(vec) => match &vec[0] {
            Value::Symbol(Symbol(s)) if s == "slide" => {
                let slide = Slide::from(value.clone());
                vec![slide]
            }
            Value::Symbol(Symbol(s)) if s == "song" => {
                vec![Slide::default()]
            }
            Value::Symbol(Symbol(s)) if s == "load" => {
                vec![Slide::default()]
            }
            _ => todo!(),
        },
        _ => todo!(),
    }
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use crate::{Background, SlideBuilder, TextAlignment};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parsing_lisp() {
        let lisp =
            read_to_string("./test_presentation.lisp").expect("oops");
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
}
