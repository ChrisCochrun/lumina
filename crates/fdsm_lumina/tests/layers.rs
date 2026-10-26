//! API and source-level tests for the multi-pass render contract. These tests
//! require no window/GPU; wgpu validates the WGSL when TextRenderer is created.
use fdsm_lumina::RenderLayer;

#[test]
fn effect_order_matches_klyff_layering_with_combined_inner_effects() {
    assert_eq!(
        RenderLayer::ORDER,
        [
            RenderLayer::Shadow,
            RenderLayer::OuterGlow,
            RenderLayer::OuterStroke,
            RenderLayer::Body,
        ]
    );
}

#[test]
fn fragment_entry_points_match_the_four_pipeline_passes() {
    let source = include_str!("../src/msdf.wgsl");
    for entry in ["fs_shadow", "fs_outer_glow", "fs_outer_stroke", "fs_body"] {
        assert!(
            source.contains(&format!("@fragment fn {entry}(")),
            "missing {entry}"
        );
    }
    assert!(
        !source.contains("fs_foreground"),
        "foreground must be split into layers"
    );
}

#[test]
fn body_contains_inner_effects_but_not_outer_effects() {
    let source = include_str!("../src/msdf.wgsl");
    let body = source
        .split("@fragment fn fs_body(")
        .nth(1)
        .expect("body shader");
    for symbol in ["style.fill", "style.stroke_in", "style.glow_in"] {
        assert!(body.contains(symbol), "body should use {symbol}");
    }
    for symbol in ["style.stroke_out", "style.glow_out", "style.shadow"] {
        assert!(!body.contains(symbol), "body must not draw {symbol}");
    }
}
