//! Copy/adapt into Lumina's slide module (this file is NOT compiled by the
//! independent renderer crate because it depends on Lumina's TextProperties).
//! The existing Slide::draw background/video/animation code is unchanged.
use std::sync::{Arc,Mutex};
use cosmic::iced::{Rectangle,core as iced_core};
use fdsm_iced_renderer::{cosmic_text::{Attrs,Family,FontSystem,Metrics},StyledTextBuilder,TextRegion};
use fdsm_iced_renderer::style::{Color as MsdfColor,GlyphColoring,Outline,Shadow,TextStyle};
use fdsm_iced_renderer::iced_adapter::FdsmTextPrimitive;
use crate::core::slide::TextProperties;

pub(crate) fn build_fdsm_primitive(
    text:&str,
    properties:&TextProperties,
    layout_bounds:Rectangle,
    font_system:Arc<Mutex<FontSystem>>,
)->Option<FdsmTextPrimitive> {
    if text.is_empty()||layout_bounds.width<=0.||layout_bounds.height<=0.{return None;}
    let mut fs=font_system.lock().ok()?;
    let scale=layout_bounds.width/1920.0;
    let base_size=properties.font_size.filter(|&v|v>0.).unwrap_or(100.);
    let metrics=Metrics::relative(base_size*scale,1.3);
    let mut attrs=Attrs::new()
        .family(properties.font.as_ref().map_or(Family::SansSerif,|f|Family::Name(f)))
        .metrics(metrics);
    if let Some(value)=properties.font_style {
        let value=match value {
            iced_core::font::Style::Normal=>fontdb::Style::Normal,
            iced_core::font::Style::Italic=>fontdb::Style::Italic,
            iced_core::font::Style::Oblique=>fontdb::Style::Oblique,
        };
        attrs=attrs.style(value);
    }
    if let Some(value)=properties.font_weight {
        let value=match value {
            iced_core::font::Weight::Thin=>fontdb::Weight::THIN,
            iced_core::font::Weight::ExtraLight=>fontdb::Weight::EXTRA_LIGHT,
            iced_core::font::Weight::Light=>fontdb::Weight::LIGHT,
            iced_core::font::Weight::Normal=>fontdb::Weight::NORMAL,
            iced_core::font::Weight::Medium=>fontdb::Weight::MEDIUM,
            iced_core::font::Weight::Semibold=>fontdb::Weight::SEMIBOLD,
            iced_core::font::Weight::Bold=>fontdb::Weight::BOLD,
            iced_core::font::Weight::ExtraBold=>fontdb::Weight::EXTRA_BOLD,
            iced_core::font::Weight::Black=>fontdb::Weight::BLACK,
        };
        attrs=attrs.weight(value);
    }
    if let Some(value)=properties.font_stretch {
        let value=match value {
            iced_core::font::Stretch::UltraCondensed=>fontdb::Stretch::UltraCondensed,
            iced_core::font::Stretch::ExtraCondensed=>fontdb::Stretch::ExtraCondensed,
            iced_core::font::Stretch::Condensed=>fontdb::Stretch::Condensed,
            iced_core::font::Stretch::SemiCondensed=>fontdb::Stretch::SemiCondensed,
            iced_core::font::Stretch::Normal=>fontdb::Stretch::Normal,
            iced_core::font::Stretch::SemiExpanded=>fontdb::Stretch::SemiExpanded,
            iced_core::font::Stretch::Expanded=>fontdb::Stretch::Expanded,
            iced_core::font::Stretch::ExtraExpanded=>fontdb::Stretch::ExtraExpanded,
            iced_core::font::Stretch::UltraExpanded=>fontdb::Stretch::UltraExpanded,
        };
        attrs=attrs.stretch(value);
    }

    let main=properties.text_color.unwrap_or_default();
    let stroke=properties.stroke_color.unwrap_or_default();
    let shadow=properties.shadow_color.unwrap_or_default();
    let offset=properties.shadow_offset.unwrap_or_default();
    let style=TextStyle {
        color:GlyphColoring::Solid(MsdfColor::rgb(main.red,main.green,main.blue)),
        stroke_out:Outline {
            color:GlyphColoring::Solid(MsdfColor::rgb(stroke.red,stroke.green,stroke.blue)),
            width:properties.stroke_size.unwrap_or_default()*scale*2.,
            roundness:1.,
        },
        shadow:Shadow {
            color:GlyphColoring::Solid(MsdfColor::rgb(shadow.red,shadow.green,shadow.blue)),
            // Klyff passed this unscaled. Keep that convention for parity.
            direction:[offset.0,offset.1],
            additional_width:properties.shadow_size.unwrap_or_default()*scale*2.,
            spread:0.3,
            roundness:1.,
        },
        ..TextStyle::default()
    };
    let mut builder=StyledTextBuilder::new(TextRegion::new(layout_bounds.width,layout_bounds.height),metrics,&attrs);
    builder.set_align(Some(properties.text_alignment.into()));
    builder.push_text(text,&attrs,style);
    let laid_out=builder.finish(&mut fs);
    drop(fs);
    let mut primitive=FdsmTextPrimitive::new(font_system);
    primitive.push_text(laid_out);
    Some(primitive)
}

/*
In Slide::draw, replace the Klyff block (keep all your animation calculations):

    let Some(text_primitive)=build_fdsm_primitive(
        &self.slide.text,
        &self.slide.text_properties.clone().unwrap_or_default(),
        bounds,
        Arc::clone(&self.settings.font_system),
    ) else {return};
    let text_height=text_primitive.total_height();
    // Keep your existing final_y / final_bounds calculation, using text_height
    // instead of querying text_primitive.text.buffer().layout_runs().
    renderer.with_layer(clip_bounds, |renderer| {
        renderer.draw_primitive(final_bounds, text_primitive);
    });
*/
