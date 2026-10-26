use fdsm_lumina::style::{Color, Glow, GlyphColoring, Outline, TextStyle};
use fdsm_lumina::{
    StyledTextBuilder, TextRegion,
    cosmic_text::{Attrs, FontSystem, Metrics},
};

#[test]
fn public_rich_text_builder_survives_multiple_spans() {
    // Uses cosmic-text's system font DB; the API must work even if the system
    // has no glyph outlines. The test does not require GPU availability.
    let mut fonts = FontSystem::new();
    let attrs = Attrs::new();
    let mut builder = StyledTextBuilder::new(
        TextRegion::new(800., 400.),
        Metrics::new(48., 56.),
        &attrs,
    );
    assert_eq!(builder.push_text("Hello ", &attrs, TextStyle::default()), 0);
    assert_eq!(
        builder.push_text(
            "world",
            &attrs,
            TextStyle {
                color: GlyphColoring::Solid(Color::rgb(1., 0., 0.)),
                stroke_out: Outline {
                    width: 4.,
                    ..Outline::default()
                },
                glow_out: Glow {
                    width: 12.,
                    spread: 0.35,
                    ..Glow::default()
                },
                ..TextStyle::default()
            }
        ),
        1
    );
    let text = builder.finish(&mut fonts);
    assert_eq!(text.styles().count(), 2);
    assert!((text.style_at(1).stroke_out.width - 4.).abs() < 1e-6);
    assert!(text.height().is_finite());
}
