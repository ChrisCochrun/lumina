use lexpr::Value;
use strum_macros::EnumString;

#[derive(Debug, Clone, Default, PartialEq, Eq, EnumString)]
pub(crate) enum Symbol {
    #[strum(ascii_case_insensitive)]
    Slide,
    #[strum(ascii_case_insensitive)]
    Image,
    #[strum(ascii_case_insensitive)]
    Text,
    #[strum(ascii_case_insensitive)]
    Video,
    #[strum(ascii_case_insensitive)]
    Song,
    #[strum(disabled)]
    ImageFit(ImageFit),
    #[strum(disabled)]
    VerseOrder(VerseOrder),
    #[strum(disabled)]
    #[default]
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
pub(crate) enum Keyword {
    ImageFit(ImageFit),
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
pub(crate) enum ImageFit {
    #[strum(ascii_case_insensitive)]
    Cover,
    #[strum(ascii_case_insensitive)]
    Fill,
    #[strum(ascii_case_insensitive)]
    Crop,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, EnumString)]
pub(crate) enum VerseOrder {
    #[strum(ascii_case_insensitive)]
    #[default]
    V1,
    #[strum(ascii_case_insensitive)]
    V2,
    #[strum(ascii_case_insensitive)]
    V3,
    #[strum(ascii_case_insensitive)]
    V4,
    #[strum(ascii_case_insensitive)]
    V5,
    #[strum(ascii_case_insensitive)]
    V6,
    #[strum(ascii_case_insensitive)]
    C1,
    #[strum(ascii_case_insensitive)]
    C2,
    #[strum(ascii_case_insensitive)]
    C3,
    #[strum(ascii_case_insensitive)]
    C4,
    #[strum(ascii_case_insensitive)]
    B1,
    #[strum(ascii_case_insensitive)]
    B2,
    #[strum(ascii_case_insensitive)]
    B3,
    #[strum(ascii_case_insensitive)]
    B4,
    #[strum(ascii_case_insensitive)]
    O1,
    #[strum(ascii_case_insensitive)]
    O2,
    #[strum(ascii_case_insensitive)]
    O3,
    #[strum(ascii_case_insensitive)]
    O4,
    #[strum(ascii_case_insensitive)]
    E1,
    #[strum(ascii_case_insensitive)]
    E2,
    #[strum(ascii_case_insensitive)]
    I1,
    #[strum(ascii_case_insensitive)]
    I2,
}

#[derive(Clone, Debug, PartialEq, Eq, EnumString)]
pub(crate) enum SongKeyword {
    #[strum(ascii_case_insensitive)]
    Title,
    #[strum(ascii_case_insensitive)]
    Author,
    #[strum(ascii_case_insensitive)]
    Ccli,
    #[strum(ascii_case_insensitive)]
    Audio,
    #[strum(ascii_case_insensitive)]
    Font,
    #[strum(ascii_case_insensitive)]
    FontSize,
    #[strum(ascii_case_insensitive)]
    Background,
    #[strum(ascii_case_insensitive)]
    VerseOrder(VerseOrder),
}

#[derive(Clone, Debug, PartialEq, Eq, EnumString)]
pub(crate) enum ImageKeyword {
    #[strum(ascii_case_insensitive)]
    Source,
    #[strum(ascii_case_insensitive)]
    Fit,
}

#[derive(Clone, Debug, Eq, PartialEq, EnumString)]
pub(crate) enum VideoKeyword {
    #[strum(ascii_case_insensitive)]
    Source,
    #[strum(ascii_case_insensitive)]
    Fit,
}

pub(crate) fn get_lists(exp: &Value) -> Vec<Value> {
    if exp.is_cons() {
        exp.as_cons().unwrap().to_vec().0
    } else {
        vec![]
    }
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use lexpr::{parse::Options, Parser};
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_list() {
        let lisp = read_to_string("./test_presentation.lisp").expect("oops");
        println!("{lisp}");
        let mut parser = Parser::from_str_custom(&lisp, Options::elisp());
        for atom in parser.value_iter() {
            match atom {
                Ok(atom) => {
                    println!("{atom}");
                    let lists = get_lists(&atom);
                    assert_eq!(lists, vec![Value::Null])
                }
                Err(e) => {
                    panic!("{e}");
                }
            }
        }
    }
}
