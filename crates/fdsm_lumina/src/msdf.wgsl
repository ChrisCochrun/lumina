
// RGB MSDF styled text shader.
//
// Text instances may contain multiple styles.
//
// Rendering is divided into four passes:
//   1. Shadows
//   2. Outer glows
//   3. Outer strokes
//   4. Glyph fill, inner glow, and inner stroke
//
// GPU struct layouts mirror src/style.rs.
// All material fields use vec4-aligned storage.

// -----------------------------------------------------------------------------
// GPU style definitions
// -----------------------------------------------------------------------------

struct ColorSpec {
    primary: vec4<f32>,
    secondary: vec4<f32>,

    // xy = gradient direction
    // z  = color mode (0 = solid, 1 = gradient)
    dir_mode: vec4<f32>,

    // xy = region origin
    // zw = region dimensions
    region: vec4<f32>,
};

struct OutlineSpec {
    color: ColorSpec,

    // x = width
    // y = roundness
    params: vec4<f32>,
};

struct GlowSpec {
    color: ColorSpec,

    // x = width
    // y = spread
    // z = roundness
    params: vec4<f32>,
};

struct ShadowSpec {
    color: ColorSpec,

    // xy = shadow direction / offset
    // z  = shadow width
    // w  = spread
    params: vec4<f32>,

    // x = roundness
    // y = softness
    extra: vec4<f32>,
};

struct TextStyle {
    fill: ColorSpec,
    stroke_out: OutlineSpec,
    stroke_in: OutlineSpec,
    glow_out: GlowSpec,
    glow_in: GlowSpec,
    shadow: ShadowSpec,
};

// -----------------------------------------------------------------------------
// GPU resources
// -----------------------------------------------------------------------------

@group(0) @binding(0)
var glyph_atlas: texture_2d<f32>;

@group(0) @binding(1)
var glyph_atlas_sampler: sampler;

@group(0) @binding(2)
var<storage, read> text_styles: array<TextStyle>;

// xy = window-space pixel origin of the Iced primitive viewport
// zw = viewport dimensions in physical pixels
@group(0) @binding(3)
var<uniform> viewport: vec4<f32>;

// -----------------------------------------------------------------------------
// Vertex input / output
// -----------------------------------------------------------------------------

struct VertexInput {
    @location(0) clip_position: vec2<f32>,
    @location(1) atlas_uv: vec2<f32>,
    @location(2) style_index: u32,
    @location(3) atlas_uv_min: vec2<f32>,
    @location(4) atlas_uv_max: vec2<f32>,
    @location(5) full_distance_range: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) atlas_uv: vec2<f32>,
    @location(1) @interpolate(flat) style_index: u32,
    @location(2) @interpolate(flat) atlas_uv_min: vec2<f32>,
    @location(3) @interpolate(flat) atlas_uv_max: vec2<f32>,
    @location(4) @interpolate(flat) full_distance_range: f32,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    output.position = vec4<f32>(input.clip_position, 0.0, 1.0);
    output.atlas_uv = input.atlas_uv;
    output.style_index = input.style_index;
    output.atlas_uv_min = input.atlas_uv_min;
    output.atlas_uv_max = input.atlas_uv_max;
    output.full_distance_range = input.full_distance_range;

    return output;
}

// -----------------------------------------------------------------------------
// MSDF utilities
// -----------------------------------------------------------------------------

/// Median of the RGB distance channels.
fn median3(channels: vec3<f32>) -> f32 {
    return max(
        min(channels.r, channels.g),
        min(max(channels.r, channels.g), channels.b)
    );
}

/// Evaluate either a solid color or a linear gradient.
///
/// Gradient positions are evaluated relative to the specified region.
/// If no region is provided, the primitive viewport dimensions are used.
fn evaluate_color(
    color_spec: ColorSpec,
    screen_position: vec2<f32>,
) -> vec4<f32> {
    let is_gradient = color_spec.dir_mode.z >= 0.5;

    if !is_gradient {
        return color_spec.primary;
    }

    var gradient_origin = vec2<f32>(0.0, 0.0);
    var gradient_size = viewport.zw;

    let has_custom_region =
        color_spec.region.z > 0.0 &&
        color_spec.region.w > 0.0;

    if has_custom_region {
        gradient_origin = color_spec.region.xy;
        gradient_size = color_spec.region.zw;
    }

    let normalized_position =
        (screen_position - gradient_origin) /
        max(gradient_size, vec2<f32>(1.0, 1.0));

    let gradient_direction = color_spec.dir_mode.xy;

    let gradient_half_span = max(
        0.5 * (
            abs(gradient_direction.x) +
            abs(gradient_direction.y)
        ),
        0.0001
    );

    let gradient_position = clamp(
        dot(
            normalized_position - vec2<f32>(0.5, 0.5),
            gradient_direction
        ) / gradient_half_span + 0.5,
        0.0,
        1.0
    );

    return mix(
        color_spec.primary,
        color_spec.secondary,
        gradient_position
    );
}

/// Convert a signed distance into antialiased coverage.
///
/// Positive distances are inside the glyph.
///
/// `roundness` interpolates between a linear coverage transition and
/// a smoothstep-shaped transition.
fn distance_coverage(
    signed_distance: f32,
    antialias_width: f32,
    roundness: f32,
) -> f32 {
    let normalized_coverage = clamp(
        signed_distance / max(antialias_width, 0.001) + 0.5,
        0.0,
        1.0
    );

    let smooth_coverage =
        normalized_coverage *
        normalized_coverage *
        (3.0 - 2.0 * normalized_coverage);

    return mix(
        normalized_coverage,
        smooth_coverage,
        clamp(roundness, 0.0, 1.0)
    );
}

/// Calculate glow opacity based on distance from the effect boundary.
///
/// `spread` determines the fraction of the glow width that remains
/// fully opaque before fading toward zero.
fn glow_coverage(
    distance_from_edge: f32,
    glow_width: f32,
    glow_spread: f32,
    glow_roundness: f32,
) -> f32 {
    if glow_width <= 0.0 {
        return 0.0;
    }

    let clamped_distance = max(distance_from_edge, 0.0);

    let solid_core_fraction = clamp(
        glow_spread,
        0.0,
        0.999
    );

    let linear_coverage = clamp(
        (glow_width - clamped_distance) /
        max(glow_width * (1.0 - solid_core_fraction), 0.001),
        0.0,
        1.0
    );

    let smooth_coverage =
        linear_coverage *
        linear_coverage *
        (3.0 - 2.0 * linear_coverage);

    return mix(
        linear_coverage,
        smooth_coverage,
        clamp(glow_roundness, 0.0, 1.0)
    );
}

/// Alpha-composite a source color over a destination color.
///
/// Inputs and output use straight (non-premultiplied) alpha.
fn composite_over(
    destination: vec4<f32>,
    source: vec4<f32>,
    coverage: f32,
) -> vec4<f32> {
    let source_alpha = clamp(
        source.a * coverage,
        0.0,
        1.0
    );

    let output_alpha =
        source_alpha +
        destination.a * (1.0 - source_alpha);

    let premultiplied_rgb =
        source.rgb * source_alpha +
        destination.rgb *
        destination.a *
        (1.0 - source_alpha);

    return vec4<f32>(
        premultiplied_rgb / max(output_alpha, 0.00001),
        output_alpha
    );
}

// -----------------------------------------------------------------------------
// MSDF distance reconstruction
// -----------------------------------------------------------------------------

struct GlyphField {
    signed_distance: f32,
    antialias_width: f32,
    fill_coverage: f32,
    screen_position: vec2<f32>,
};

/// Reconstruct the MSDF distance scale in screen pixels.
///
/// Every glyph carries its own full encoded distance range because
/// different glyphs can have different atlas padding.
fn calculate_screen_distance_range(
    atlas_uv: vec2<f32>,
    full_distance_range: f32,
) -> f32 {
    let atlas_dimensions =
        vec2<f32>(textureDimensions(glyph_atlas, 0));

    let atlas_uv_derivatives = max(
        fwidth(atlas_uv),
        vec2<f32>(0.000001)
    );

    let normalized_distance_range =
        vec2<f32>(full_distance_range) / atlas_dimensions;

    return max(
        0.5 * dot(
            normalized_distance_range,
            1.0 / atlas_uv_derivatives
        ),
        1.0
    );
}

/// Sample a glyph and reconstruct its signed distance and coverage.
///
/// The RGB median represents the distance to the glyph outline.
/// Positive values are inside the glyph.
fn evaluate_glyph_field(input: VertexOutput) -> GlyphField {
    let screen_distance_range = calculate_screen_distance_range(
        input.atlas_uv,
        input.full_distance_range
    );

    let sampled_distance = textureSample(
        glyph_atlas,
        glyph_atlas_sampler,
        input.atlas_uv
    );

    let signed_distance =
        (median3(sampled_distance.rgb) - 0.5) *
        screen_distance_range;

    let antialias_width = max(
        fwidth(signed_distance),
        0.75
    );

    let fill_coverage = distance_coverage(
        signed_distance,
        antialias_width,
        0.5
    );

    let screen_position =
        input.position.xy - viewport.xy;

    return GlyphField(
        signed_distance,
        antialias_width,
        fill_coverage,
        screen_position
    );
}

// -----------------------------------------------------------------------------
// Pass 1: Shadow
// -----------------------------------------------------------------------------
//
// All glyph shadows must be drawn before any glyph foregrounds.
// This prevents later glyph shadows from covering earlier glyph fills.

@fragment
fn fs_shadow(input: VertexOutput) -> @location(0) vec4<f32> {
    let shadow_style = text_styles[input.style_index].shadow;

    let screen_distance_range = calculate_screen_distance_range(
        input.atlas_uv,
        input.full_distance_range
    );

    // Convert the shadow's screen-pixel offset into atlas UV coordinates.
    let shadow_atlas_offset =
        dpdx(input.atlas_uv) * shadow_style.params.x +
        dpdy(input.atlas_uv) * shadow_style.params.y;

    // Clamp to this glyph's atlas rectangle to prevent samples from
    // reading into adjacent glyphs.
    let shadow_sample_uv = clamp(
        input.atlas_uv - shadow_atlas_offset,
        input.atlas_uv_min,
        input.atlas_uv_max
    );

    let shadow_distance_sample = textureSample(
        glyph_atlas,
        glyph_atlas_sampler,
        shadow_sample_uv
    );

    let shadow_signed_distance =
        (median3(shadow_distance_sample.rgb) - 0.5) *
        screen_distance_range;

    let shadow_antialias_width = max(
        fwidth(shadow_signed_distance),
        0.75
    );

    // Shadow params:
    //   x/y = offset
    //   z   = expansion width
    //   w   = spread
    //
    // Shadow extra:
    //   x = roundness
    //   y = independent softness
    let effective_shadow_softness = max(
        shadow_antialias_width,
        shadow_style.extra.y * (1.0 - shadow_style.params.w)
    );

    let shadow_coverage = distance_coverage(
        shadow_signed_distance + shadow_style.params.z,
        effective_shadow_softness,
        shadow_style.extra.x
    );

    let screen_position =
        input.position.xy - viewport.xy;

    return composite_over(
        vec4<f32>(0.0, 0.0, 0.0, 0.0),
        evaluate_color(shadow_style.color, screen_position),
        shadow_coverage
    );
}

// -----------------------------------------------------------------------------
// Pass 2: Outer glow
// -----------------------------------------------------------------------------
//
// Complete the outer-glow pass for all glyphs before rendering strokes
// or glyph bodies.

@fragment
fn fs_outer_glow(input: VertexOutput) -> @location(0) vec4<f32> {
    let text_style = text_styles[input.style_index];
    let glyph_field = evaluate_glyph_field(input);

    let outer_glow = text_style.glow_out;

    let outside_glyph_coverage =
        1.0 - glyph_field.fill_coverage;

    // Glow starts outside the outer-stroke boundary.
    let distance_beyond_outer_stroke = max(
        -glyph_field.signed_distance -
        text_style.stroke_out.params.x,
        0.0
    );

    let outer_glow_coverage = glow_coverage(
        distance_beyond_outer_stroke,
        outer_glow.params.x,
        outer_glow.params.y,
        outer_glow.params.z
    ) * outside_glyph_coverage;

    return composite_over(
        vec4<f32>(0.0, 0.0, 0.0, 0.0),
        evaluate_color(
            outer_glow.color,
            glyph_field.screen_position
        ),
        outer_glow_coverage
    );
}

// -----------------------------------------------------------------------------
// Pass 3: Outer stroke
// -----------------------------------------------------------------------------
//
// All outer strokes render before glyph bodies. Tight letter spacing
// therefore cannot cause a later glyph's stroke to cover an earlier fill.

@fragment
fn fs_outer_stroke(input: VertexOutput) -> @location(0) vec4<f32> {
    let outer_stroke =
        text_styles[input.style_index].stroke_out;

    let glyph_field = evaluate_glyph_field(input);

    let expanded_glyph_coverage = distance_coverage(
        glyph_field.signed_distance + outer_stroke.params.x,
        glyph_field.antialias_width,
        outer_stroke.params.y
    );

    let outer_stroke_coverage = max(
        0.0,
        expanded_glyph_coverage - glyph_field.fill_coverage
    );

    return composite_over(
        vec4<f32>(0.0, 0.0, 0.0, 0.0),
        evaluate_color(
            outer_stroke.color,
            glyph_field.screen_position
        ),
        outer_stroke_coverage
    );
}

// -----------------------------------------------------------------------------
// Pass 4: Glyph body + inner glow + inner stroke
// -----------------------------------------------------------------------------
//
// The body and its inner effects are composited in one fragment invocation.
// All inner effects remain restricted to the glyph interior.

@fragment
fn fs_body(input: VertexOutput) -> @location(0) vec4<f32> {
    let text_style = text_styles[input.style_index];
    let glyph_field = evaluate_glyph_field(input);

    // Base glyph fill.
    var output_color = composite_over(
        vec4<f32>(0.0, 0.0, 0.0, 0.0),
        evaluate_color(
            text_style.fill,
            glyph_field.screen_position
        ),
        glyph_field.fill_coverage
    );

    let inner_stroke = text_style.stroke_in;
    let inner_glow = text_style.glow_in;

    // Inner glow begins beyond the inner-stroke boundary.
    let distance_beyond_inner_stroke = max(
        glyph_field.signed_distance - inner_stroke.params.x,
        0.0
    );

    let inner_glow_coverage = glow_coverage(
        distance_beyond_inner_stroke,
        inner_glow.params.x,
        inner_glow.params.y,
        inner_glow.params.z
    ) * glyph_field.fill_coverage;

    output_color = composite_over(
        output_color,
        evaluate_color(
            inner_glow.color,
            glyph_field.screen_position
        ),
        inner_glow_coverage
    );

    // Inner stroke occupies the region between the glyph boundary
    // and the inward-offset boundary.
    let inner_stroke_boundary_coverage = distance_coverage(
        glyph_field.signed_distance - inner_stroke.params.x,
        glyph_field.antialias_width,
        inner_stroke.params.y
    );

    let inner_stroke_coverage = max(
        0.0,
        glyph_field.fill_coverage -
        inner_stroke_boundary_coverage
    );

    output_color = composite_over(
        output_color,
        evaluate_color(
            inner_stroke.color,
            glyph_field.screen_position
        ),
        inner_stroke_coverage
    );

    return output_color;
}
