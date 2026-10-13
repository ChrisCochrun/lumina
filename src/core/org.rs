use std::{fs, path::Path};

use cosmic::{
    cosmic_theme::palette::Srgb,
    iced::font::{Style, Weight},
};
use orgize::{
    Org,
    export::{Container, Event, Traverser},
};

use crate::core::{
    images::Image,
    slide::{Background, Slide, SlideBuilder},
    songs::Song,
    videos::Video,
};

pub struct OrgSlide {
    heading: OrgSlideText,
    content: OrgTextContent,
    background: Background,
    // animtations
    // transitions
}

struct OrgSlideText {
    pub text: String,
    pub properties: OrgTextProperties,
}

struct OrgTextProperties {
    pub font: Option<String>,
    pub font_size: Option<u16>,
    pub font_weight: Option<Weight>,
    pub font_style: Option<Style>,
    pub text_color: Option<Srgb>,
    pub stroke_size: Option<u16>,
    pub stroke_color: Option<Srgb>,
    pub shadow_size: Option<u16>,
    pub shadow_offset: Option<(i16, i16)>,
    pub shadow_color: Option<Srgb>,
}

enum OrgTextContent {
    Text(OrgSlideText),
    List {
        list: Vec<String>,
        properties: OrgTextProperties,
    },
    Quote(OrgSlideText),
}

pub struct OrgService {
    items: Vec<OrgServiceItem>,
}

pub enum OrgServiceItem {
    Song(Song),
    Video(Video),
    Image(Image),
    Slides(Vec<OrgSlide>),
    Slide(OrgSlide),
}

impl OrgSlide {
    pub fn to_slide(&self) -> Slide {
        todo!()
    }
}

#[derive(Default)]
struct LumTraverser {
    slides: Vec<Slide>,
    in_list: Vec<bool>,
    current_slide: Option<SlideBuilder>,
}

impl Traverser for LumTraverser {
    fn event(
        &mut self,
        event: orgize::export::Event,
        ctx: &mut orgize::export::TraversalContext,
    ) {
        match event {
            Event::Enter(Container::Headline(headline)) => {
                let slide_text = headline
                    .title()
                    .map(|title| title.to_string())
                    .collect::<String>();
                let slide = SlideBuilder::new().text(slide_text);
                self.current_slide = Some(slide);
            }
            Event::Leave(Container::Headline(_)) => {
                if let Some(slide) = self
                    .current_slide
                    .to_owned()
                    .map(|builder| builder.build().ok())
                    .flatten()
                {
                    self.slides.push(slide);
                    self.current_slide = None;
                }
            }
            Event::Enter(Container::Section(section)) => {
                todo!()
            }
            Event::Enter(Container::Keyword(keyword)) => {
                if keyword.key().to_uppercase() == "LUMINA" {
                    let value = keyword.value();
                    match value.to_lowercase().as_str() {
                        "song:" => todo!(),
                        "video:" => todo!(),
                        "image:" => todo!(),
                        "background:" => todo!(),
                        _ => todo!(),
                    }
                }
            }
            orgize::export::Event::Text(token) => todo!(),
            orgize::export::Event::LineBreak(line_break) => todo!(),
            orgize::export::Event::Entity(entity) => todo!(),
            _ => {}
        }
    }
}

impl OrgService {
    pub fn new(path: impl AsRef<Path>) -> OrgService {
        let org_doc = Org::parse(fs::read_to_string(path).unwrap_or("".to_string()));
        let mut traverser = LumTraverser::default();
        org_doc.traverse(&mut traverser);
        todo!()
    }
}
