// RGB MSDF styled text shader. One text instance can have many styles.
// GPU layout mirrors src/style.rs (only vec4-aligned fields).
struct ColorSpec {
    primary: vec4<f32>,
    secondary: vec4<f32>,
    dir_mode: vec4<f32>,
    region: vec4<f32>,
};
struct OutlineSpec {color: ColorSpec, params: vec4<f32>};
struct GlowSpec {color: ColorSpec, params: vec4<f32>};
struct ShadowSpec {color: ColorSpec, params: vec4<f32>, extra: vec4<f32>};
struct TextStyle {
    fill: ColorSpec,
    stroke_out: OutlineSpec,
    stroke_in: OutlineSpec,
    glow_out: GlowSpec,
    glow_in: GlowSpec,
    shadow: ShadowSpec,
};
@group(0) @binding(0) var atlas: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler;
@group(0) @binding(2) var<storage, read> styles: array<TextStyle>;
// x,y = window-space pixel origin of Iced's primitive viewport;
// z,w = viewport width,height in physical pixels.
@group(0) @binding(3) var<uniform> viewport: vec4<f32>;

struct VertexIn {
    @location(0) clip: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) index: u32,
    @location(3) uv_min: vec2<f32>,
    @location(4) uv_max: vec2<f32>,
    @location(5) full_range: f32,
};
struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) index: u32,
    @location(2) @interpolate(flat) uv_min: vec2<f32>,
    @location(3) @interpolate(flat) uv_max: vec2<f32>,
    @location(4) @interpolate(flat) full_range: f32,
};
@vertex fn vs_main(v:VertexIn)->VertexOut{
    var o:VertexOut;
    o.clip=vec4<f32>(v.clip,0.,1.);
    o.uv=v.uv;
    o.index=v.index;
    o.uv_min=v.uv_min;
    o.uv_max=v.uv_max;
    o.full_range=v.full_range;
    return o;
}
fn median3(c:vec3<f32>)->f32{
    return max(min(c.r,c.g),min(max(c.r,c.g),c.b));
}
fn paint(p:ColorSpec,pixel:vec2<f32>)->vec4<f32>{
    if p.dir_mode.z<0.5 {return p.primary;}
    var origin=vec2<f32>(0.,0.);
    var size=viewport.zw;
    if p.region.z>0. && p.region.w>0. {origin=p.region.xy;size=p.region.zw;}
    let q=(pixel-origin)/max(size,vec2<f32>(1.,1.));
    let direction=p.dir_mode.xy;
    let span=max(0.5*(abs(direction.x)+abs(direction.y)),0.0001);
    let t=clamp(dot(q-vec2<f32>(0.5,0.5),direction)/span+0.5,0.,1.);
    return mix(p.primary,p.secondary,t);
}
fn cov(d:f32,aa:f32,roundness:f32)->f32{
    let t=clamp(d/max(aa,0.001)+0.5,0.,1.);
    let rounded=t*t*(3.-2.*t);
    return mix(t,rounded,clamp(roundness,0.,1.));
}
fn glow_curve(distance_from_edge:f32,width:f32,spread:f32,roundness:f32)->f32{
    if width<=0. {return 0.;}
    let distance=max(distance_from_edge,0.);
    // `spread` is 0..1 as in Klyff: solid core until width*spread.
    let core=clamp(spread,0.,0.999);
    let linear=clamp((width-distance)/max(width*(1.-core),0.001),0.,1.);
    let sm=linear*linear*(3.-2.*linear);
    return mix(linear,sm,clamp(roundness,0.,1.));
}
fn over(dst:vec4<f32>,source:vec4<f32>,coverage:f32)->vec4<f32>{
    let a=clamp(source.a*coverage,0.,1.);
    let out_a=a+dst.a*(1.-a);
    let rgb=source.rgb*a+dst.rgb*dst.a*(1.-a);
    return vec4<f32>(rgb/max(out_a,0.00001),out_a);
}
@fragment fn fs_main(v:VertexOut)->@location(0) vec4<f32>{
    let style=styles[v.index];
    let atlas_wh=vec2<f32>(textureDimensions(atlas,0));
    let uv_grad=max(fwidth(v.uv),vec2<f32>(0.000001));
    // full_range is PER GLYPH. It cannot be a single uniform with a
    // cache that permits different padding for different glyphs.
    let screen_range=max(0.5*dot(vec2<f32>(v.full_range)/atlas_wh,1./uv_grad),1.);
    let texel=textureSample(atlas,atlas_sampler,v.uv);
    let d=(median3(texel.rgb)-0.5)*screen_range; // positive = inside glyph
    let aa=max(fwidth(d),0.75);
    let screen_pos=v.clip.xy-viewport.xy; // gradient coordinates local to Iced viewport
    var output=vec4<f32>(0.,0.,0.,0.);

    let shadow=style.shadow;
    let uv_offset=dpdx(v.uv)*shadow.params.x+dpdy(v.uv)*shadow.params.y;
    let suv=clamp(v.uv-uv_offset,v.uv_min,v.uv_max);
    let sd=(median3(textureSample(atlas,atlas_sampler,suv).rgb)-0.5)*screen_range;
    let shadow_softness=max(aa,shadow.params.z*(1.-shadow.params.w));
    let shcov=cov(sd+shadow.params.z,shadow_softness,shadow.extra.x);
    output=over(output,paint(shadow.color,screen_pos),shcov);

    let fill=cov(d,aa,0.5);
    let so=style.stroke_out;
    let outside=1.-fill;
    // Outer glow begins at the outside of the outer stroke.
    let go=style.glow_out;
    let go_dist=max(-d-so.params.x,0.);
    let go_cov=glow_curve(go_dist,go.params.x,go.params.y,go.params.z)*outside;
    output=over(output,paint(go.color,screen_pos),go_cov);

    let outer_edge=cov(d+so.params.x,aa,so.params.y);
    output=over(output,paint(so.color,screen_pos),max(0.,outer_edge-fill));
    output=over(output,paint(style.fill,screen_pos),fill);

    let si=style.stroke_in;
    let gi=style.glow_in;
    // Inner glow starts after the inner stroke.
    let gi_cov=glow_curve(max(d-si.params.x,0.),gi.params.x,gi.params.y,gi.params.z)*fill;
    output=over(output,paint(gi.color,screen_pos),gi_cov);
    let inner_edge=cov(d-si.params.x,aa,si.params.y);
    output=over(output,paint(si.color,screen_pos),max(0.,fill-inner_edge));
    return output;
}
