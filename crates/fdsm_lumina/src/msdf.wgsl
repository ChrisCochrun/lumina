struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) style_index: u32,
    @location(3) uv_min: vec2<f32>,
    @location(4) uv_max: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) style_index: u32,
    @location(2) @interpolate(flat) uv_min: vec2<f32>,
    @location(3) @interpolate(flat) uv_max: vec2<f32>,
};

struct GpuColoring {
    primary: vec4<f32>,
    secondary: vec4<f32>,
    // x/y = direction, z = 0 for solid and 1 for gradient.
    direction_mode: vec4<f32>,
    // x/y/width/height; a zero-sized region means the viewport.
    region: vec4<f32>,
};

struct GpuOutline {
    color: GpuColoring,
    // x = width, y = transition shaping control.
    params: vec4<f32>,
};

struct GpuGlow {
    color: GpuColoring,
    // x = width, y = falloff exponent, z = smooth-falloff blend.
    params: vec4<f32>,
};

struct GpuShadow {
    color: GpuColoring,
    // x/y = pixel offset, z = extra width, w = approximate softness.
    params: vec4<f32>,
    // x = smooth-falloff blend.
    extra: vec4<f32>,
};

struct GpuTextStyle {
    fill: GpuColoring,
    stroke_out: GpuOutline,
    stroke_in: GpuOutline,
    glow_out: GpuGlow,
    glow_in: GpuGlow,
    shadow: GpuShadow,
};

@group(0) @binding(0)
var msdf_texture: texture_2d<f32>;

@group(0) @binding(1)
var msdf_sampler: sampler;

@group(0) @binding(2)
var<storage, read> text_styles: array<GpuTextStyle>;

// x/y are physical viewport dimensions; z is the full FDSM distance range in atlas texels.
@group(0) @binding(3)
var<uniform> viewport: vec4<f32>;

// Keep this layout in sync with the CPU's four-float viewport uniform.
const EPSILON: f32 = 0.00001;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.uv = input.uv;
    output.style_index = input.style_index;
    output.uv_min = input.uv_min;
    output.uv_max = input.uv_max;
    return output;
}

fn median3(v: vec3<f32>) -> f32 {
    return max(min(v.r, v.g), min(max(v.r, v.g), v.b));
}

fn msdf_distance_px(sample_value: vec4<f32>, screen_px_range: f32) -> f32 {
    // FDSM produces RGB MSDF. Median reconstruction is the best distance
    // estimate available to this renderer, but is not an independent true SDF.
    return (median3(sample_value.rgb) - 0.5) * screen_px_range;
}

fn edge_coverage(distance_px: f32, aa: f32, roundness: f32) -> f32 {
    // The source format has only RGB MSDF, so roundness controls transition
    // shaping rather than an independent MTSDF alpha-distance correction.
    let t = clamp(distance_px / max(aa, EPSILON) + 0.5, 0.0, 1.0);
    let smooth_t = t * t * (3.0 - 2.0 * t);
    return mix(t, smooth_t, clamp(roundness, 0.0, 1.0));
}

fn paint_color(paint: GpuColoring, screen_position: vec2<f32>) -> vec4<f32> {
    if paint.direction_mode.z < 0.5 {
        return paint.primary;
    }

    let explicit_size = paint.region.zw;
    let has_explicit_region = all(explicit_size > vec2<f32>(0.0));
    var region_origin = vec2<f32>(0.0);
    var region_size = viewport.xy;
    if has_explicit_region {
        region_origin = paint.region.xy;
        region_size = explicit_size;
    }
    let local = (screen_position - region_origin) / max(region_size, vec2<f32>(1.0));
    let direction = normalize(paint.direction_mode.xy);
    let half_span = max(0.5 * (abs(direction.x) + abs(direction.y)), EPSILON);
    let t = clamp(dot(local - vec2<f32>(0.5), direction) / half_span + 0.5, 0.0, 1.0);
    return mix(paint.primary, paint.secondary, t);
}

fn glow_coverage(
    distance_px: f32,
    width: f32,
    spread: f32,
    roundness: f32,
    outer_side: bool,
) -> f32 {
    if width <= 0.0 {
        return 0.0;
    }
    var from_edge = max(distance_px, 0.0);
    if outer_side {
        from_edge = max(-distance_px, 0.0);
    }
    let t = clamp(1.0 - from_edge / max(width, EPSILON), 0.0, 1.0);
    let power_falloff = pow(t, max(spread, 0.01));
    let smooth_falloff = t * t * (3.0 - 2.0 * t);
    return mix(power_falloff, smooth_falloff, clamp(roundness, 0.0, 1.0));
}

fn over(dst: vec4<f32>, rgb_alpha: vec4<f32>, coverage: f32) -> vec4<f32> {
    let source_alpha = clamp(rgb_alpha.a * coverage, 0.0, 1.0);
    let out_alpha = source_alpha + dst.a * (1.0 - source_alpha);
    let premultiplied_rgb = rgb_alpha.rgb * source_alpha
        + dst.rgb * dst.a * (1.0 - source_alpha);
    return vec4<f32>(premultiplied_rgb / max(out_alpha, EPSILON), out_alpha);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let style = text_styles[input.style_index];
    let atlas_size = vec2<f32>(textureDimensions(msdf_texture, 0));
    let unit_range = vec2<f32>(viewport.z) / atlas_size;
    let uv_per_pixel = max(fwidth(input.uv), vec2<f32>(EPSILON));
    // `viewport.z` is the full encoded distance range. FDSM maps samples to
    // [-range/2, range/2], so it must be twice the atlas margin.
    let screen_px_range = max(0.5 * dot(unit_range, 1.0 / uv_per_pixel), 1.0);

    let sample_value = textureSample(msdf_texture, msdf_sampler, input.uv);
    let distance_px = msdf_distance_px(sample_value, screen_px_range);
    let aa = max(fwidth(distance_px), 0.75);
    let fill_coverage = edge_coverage(distance_px, aa, 0.5);
    let screen_position = input.position.xy;

    // Shadow samples the same atlas at an offset measured in screen pixels.
    // Derivatives convert that offset to atlas UVs, independent of atlas size.
    let shadow_offset_uv = dpdx(input.uv) * style.shadow.params.x
        + dpdy(input.uv) * style.shadow.params.y;
    // Each glyph's quad carries its atlas sub-rectangle. Clamp the displaced
    // sample to this tile so shadows can never read a neighboring glyph.
    let shadow_uv = clamp(input.uv - shadow_offset_uv, input.uv_min, input.uv_max);
    let shadow_sample = textureSample(msdf_texture, msdf_sampler, shadow_uv);
    let shadow_distance = msdf_distance_px(shadow_sample, screen_px_range);
    let shadow_aa = max(aa + style.shadow.params.w, 0.75);

    var result = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    // Back-to-front compositing is done in one fragment invocation, so each
    // span can use all effects without sorting or issuing a draw per effect.
    let shadow_coverage = edge_coverage(
        shadow_distance + style.shadow.params.z,
        shadow_aa,
        style.shadow.extra.x,
    );
    result = over(result, paint_color(style.shadow.color, screen_position), shadow_coverage);

    let glow_out_coverage = glow_coverage(
        distance_px,
        style.glow_out.params.x,
        style.glow_out.params.y,
        style.glow_out.params.z,
        true,
    ) * (1.0 - fill_coverage);
    result = over(result, paint_color(style.glow_out.color, screen_position), glow_out_coverage);

    let stroke_out_width = style.stroke_out.params.x;
    if stroke_out_width > 0.0 {
        let expanded = edge_coverage(
            distance_px + stroke_out_width,
            aa,
            style.stroke_out.params.y,
        );
        let inner = edge_coverage(distance_px, aa, style.stroke_out.params.y);
        result = over(
            result,
            paint_color(style.stroke_out.color, screen_position),
            clamp(expanded - inner, 0.0, 1.0),
        );
    }

    result = over(result, paint_color(style.fill, screen_position), fill_coverage);

    let glow_in_coverage = glow_coverage(
        distance_px,
        style.glow_in.params.x,
        style.glow_in.params.y,
        style.glow_in.params.z,
        false,
    ) * fill_coverage;
    result = over(result, paint_color(style.glow_in.color, screen_position), glow_in_coverage);

    let stroke_in_width = style.stroke_in.params.x;
    if stroke_in_width > 0.0 {
        let outer = edge_coverage(distance_px, aa, style.stroke_in.params.y);
        let inner = edge_coverage(
            distance_px - stroke_in_width,
            aa,
            style.stroke_in.params.y,
        );
        result = over(
            result,
            paint_color(style.stroke_in.color, screen_position),
            clamp(outer - inner, 0.0, 1.0),
        );
    }

    return result;
}
