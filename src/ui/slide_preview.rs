use crate::Slide;

pub struct SlidePreview<'a> {
    slides: Vec<&'a Slide>,
}
