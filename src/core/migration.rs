use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    core::{
        animation::Animation,
        kinds::ServiceItemKind,
        service_items::ServiceItem,
        slide::{Background, Slide, SlideId, TextAlignment},
    },
    ui::text_svg::{Color, Font, Shadow, Stroke, TextSvg},
};

#[derive(Serialize, Deserialize)]
enum SlideSer {
    V1 {
        id: SlideId,
        background: Background,
        text: String,
        font: Option<Font>,
        font_size: i32,
        stroke: Option<Stroke>,
        shadow: Option<Shadow>,
        text_alignment: TextAlignment,
        text_color: Option<Color>,
        audio: Option<PathBuf>,
        video_loop: bool,
        video_start_time: f32,
        video_end_time: f32,
        pdf_index: u32,
        text_svg: Option<TextSvg>,
    },
}

#[derive(Serialize, Deserialize)]
enum ServiceItemSer {
    V1 {
        id: i32,
        title: String,
        database_id: i32,
        kind: ServiceItemKind,
        slides: Vec<Slide>,
        animation: Option<Animation>,
    },
}

impl Serialize for Slide {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let &Self {
            ref id,
            ref background,
            ref text,
            ref font,
            font_size,
            ref stroke,
            ref shadow,
            text_alignment,
            ref text_color,
            ref audio,
            video_loop,
            video_start_time,
            video_end_time,
            pdf_index,
            ref text_svg,
            ..
        }: &Slide = self;

        SlideSer::V1 {
            id: id.clone(),
            background: background.clone(),
            text: text.clone(),
            font: font.clone(),
            font_size,
            stroke: stroke.clone(),
            shadow: shadow.clone(),
            text_alignment,
            text_color: text_color.clone(),
            audio: audio.clone(),
            video_loop,
            video_start_time,
            video_end_time,
            pdf_index,
            text_svg: text_svg.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Slide {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match SlideSer::deserialize(deserializer)? {
            SlideSer::V1 {
                id,
                background,
                text,
                font,
                font_size,
                stroke,
                shadow,
                text_alignment,
                text_color,
                audio,
                video_loop,
                video_start_time,
                video_end_time,
                pdf_index,
                text_svg,
            } => Slide {
                id,
                background,
                text,
                font,
                font_size,
                stroke,
                shadow,
                text_alignment,
                text_color,
                audio,
                video_loop,
                video_start_time,
                video_end_time,
                pdf_index,
                text_svg,
                ..Default::default()
            },
        })
    }
}

impl Serialize for ServiceItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let &Self {
            id,
            ref title,
            database_id,
            ref kind,
            ref slides,
            ref animation,
        }: &ServiceItem = self;
        ServiceItemSer::V1 {
            id,
            title: title.clone(),
            database_id,
            kind: kind.clone(),
            slides: slides.clone(),
            animation: animation.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ServiceItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match ServiceItemSer::deserialize(deserializer)? {
            ServiceItemSer::V1 {
                id,
                title,
                database_id,
                kind,
                slides,
                animation,
            } => ServiceItem {
                id,
                title,
                database_id,
                kind,
                slides,
                animation,
            },
        })
    }
}
