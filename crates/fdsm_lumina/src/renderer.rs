//! Persistent GPU renderer. One shared atlas/pipeline; a distinct PreparedText
//! per text primitive so multiple slides can prepare before Iced draws them.
use crate::{
    atlas::{AtlasRect, CacheKey, GlyphAtlas},
    cosmic_text::FontSystem,
    style::{GpuTextStyle, GradientRegion},
    text::StyledText,
};
use bytemuck::{Pod, Zeroable};
use iced_wgpu::wgpu::{self, util::DeviceExt};

#[derive(Debug)]
pub enum RenderError {
    Font(String),
    AtlasFull { width: u32, height: u32 },
    InvalidRegion,
}
impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Font(s) => write!(f, "FDSM glyph: {s}"),
            Self::AtlasFull { width, height } => write!(
                f,
                "FDSM atlas exhausted at {width}x{height}; increase RendererConfig atlas size"
            ),
            Self::InvalidRegion => write!(f, "invalid text region or render bounds"),
        }
    }
}
impl std::error::Error for RenderError {}
#[derive(Debug, Clone, Copy)]
pub struct RendererConfig {
    pub atlas_width: u32,
    pub atlas_height: u32,
}
impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            atlas_width: 2048,
            atlas_height: 2048,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
    style_index: u32,
    uv_min: [f32; 2],
    uv_max: [f32; 2],
    distance_range: f32,
}
// One storage style table and one uniform; both in bind group 0 along with
// texture and sampler. The per-glyph range is a vertex attribute, NOT a
// global uniform (glyphs in the shared atlas can have different padding).
pub struct TextRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    atlas: GlyphAtlas,
}

#[derive(Debug)]
pub struct PreparedText {
    vertices: wgpu::Buffer,
    count: u32,
    _styles: wgpu::Buffer,
    _viewport: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}
impl PreparedText {
    pub fn vertex_count(&self) -> u32 {
        self.count
    }
}

/// Rectangle in physical surface pixels. Iced already establishes the viewport
/// and scissor for the primitive. Its `origin` is used only by gradients.
#[derive(Debug, Clone, Copy)]
pub struct PhysicalBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl PhysicalBounds {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

impl TextRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        tracing::debug!(target: "fdsm","NEW TEXT RENDERER");
        Self::with_config(device, format, RendererConfig::default())
    }
    pub fn with_config(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        config: RendererConfig,
    ) -> Self {
        let atlas = GlyphAtlas::new(device, config.atlas_width, config.atlas_height);
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fdsm styled text BGL"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("fdsm text pipeline layout"),
                bind_group_layouts: &[&bind_group_layout],
                immediate_size: 0,
            });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fdsm styled MSDF"),
            source: wgpu::ShaderSource::Wgsl(include_str!("msdf.wgsl").into()),
        });
        let attributes = wgpu::vertex_attr_array![
            0=>Float32x2,1=>Float32x2,2=>Uint32,3=>Float32x2,4=>Float32x2,5=>Float32,
        ];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fdsm styled text"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attributes,
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            bind_group_layout,
            atlas,
        }
    }
    /// Prepare a single text instance. Returns OWNED resources for that instance;
    /// no subsequent prepare() can overwrite a previously prepared instance.
    /// `font_source` distinguishes independently allocated FontSystems whose
    /// databases may reuse the same numeric fontdb IDs.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        font_system: &FontSystem,
        font_source: usize,
        text: &StyledText,
        bounds: PhysicalBounds,
    ) -> Result<Option<PreparedText>, RenderError> {
        if !bounds.width.is_finite()
            || !bounds.height.is_finite()
            || bounds.width <= 0.
            || bounds.height <= 0.
            || text.region.width <= 0.
            || text.region.height <= 0.
        {
            return Err(RenderError::InvalidRegion);
        }
        let sx = bounds.width / text.region.width * text.scale;
        let sy = bounds.height / text.region.height * text.scale;
        if !sx.is_finite() || !sy.is_finite() || sx <= 0. || sy <= 0. {
            return Err(RenderError::InvalidRegion);
        }
        let s = sx; // glyph MSDF generation uses horizontal scale; vertical quad adapts by sy/sx
        let stretch_y = sy / sx;
        let styles = text.styles.iter().map(|v| v.scaled(s)).collect::<Vec<_>>();
        let padding = ((styles
            .iter()
            .map(|v| v.required_padding())
            .fold(0.0_f32, f32::max)
            + 2.)
            .max(16.))
        .ceil() as u32;
        let region = text.gradient_region.map(|r| GradientRegion {
            x: r.x * sx,
            y: r.y * sy,
            width: r.width * sx,
            height: r.height * sy,
        });
        let gpu_styles = styles
            .into_iter()
            .map(|st| st.to_gpu(region))
            .collect::<Vec<GpuTextStyle>>();
        let mut verts = Vec::<Vertex>::new();
        for line in &text.lines {
            // When the cosmic-text buffer already aligned a line, glyph.x has
            // the appropriate horizontal offset. Do not center it a second time.
            for g in &line.glyphs {
                let key = CacheKey::new(font_source, g.key, g.font_size * s, padding);
                let info = self.atlas.obtain(queue, font_system, key)?;
                let Some(rect) = info.rect else { continue };
                let x = (g.x + g.x_offset * g.font_size) * sx + info.bearing_x;
                let y = (line.baseline + g.y - g.y_offset * g.font_size) * sy
                    - info.bearing_y * stretch_y;
                append_quad(
                    &mut verts,
                    x,
                    y,
                    rect,
                    self.atlas.width,
                    self.atlas.height,
                    bounds.width,
                    bounds.height,
                    g.style_index,
                    2. * padding as f32,
                    stretch_y,
                );
            }
        }
        if verts.is_empty() {
            return Ok(None);
        }
        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("fdsm text vertices"),
                contents: bytemuck::cast_slice(&verts),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let style_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fdsm style table"),
            contents: bytemuck::cast_slice(&gpu_styles),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let viewport = [bounds.x, bounds.y, bounds.width, bounds.height];
        let viewport_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("fdsm text viewport"),
                contents: bytemuck::cast_slice(&viewport),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fdsm per-text bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.atlas.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.atlas.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: style_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: viewport_buffer.as_entire_binding(),
                },
            ],
        });
        Ok(Some(PreparedText {
            vertices: vertex_buffer,
            count: verts.len() as u32,
            _styles: style_buffer,
            _viewport: viewport_buffer,
            bind_group,
        }))
    }
    pub fn render(&self, pass: &mut wgpu::RenderPass<'_>, prepared: &PreparedText) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &prepared.bind_group, &[]);
        pass.set_vertex_buffer(0, prepared.vertices.slice(..));
        pass.draw(0..prepared.count, 0..1);
    }
    pub fn cached_glyphs(&self) -> usize {
        self.atlas.glyphs.len()
    }
}
fn clip(x: f32, y: f32, w: f32, h: f32) -> [f32; 2] {
    [2. * x / w - 1., 1. - 2. * y / h]
}
#[allow(clippy::too_many_arguments)]
fn append_quad(
    out: &mut Vec<Vertex>,
    x: f32,
    y: f32,
    rect: AtlasRect,
    atlas_w: u32,
    atlas_h: u32,
    view_w: f32,
    view_h: f32,
    style: u32,
    range: f32,
    stretch_y: f32,
) {
    let u0 = (rect.x as f32 + 0.5) / atlas_w as f32;
    let v0 = (rect.y as f32 + 0.5) / atlas_h as f32;
    let u1 = (rect.x as f32 + rect.width as f32 - 0.5) / atlas_w as f32;
    let v1 = (rect.y as f32 + rect.height as f32 - 0.5) / atlas_h as f32;
    let x1 = x + rect.width as f32;
    let y1 = y + rect.height as f32 * stretch_y;
    let uv_min = [u0, v0];
    let uv_max = [u1, v1];
    let a = Vertex {
        position: clip(x, y, view_w, view_h),
        uv: [u0, v0],
        style_index: style,
        uv_min,
        uv_max,
        distance_range: range,
    };
    let b = Vertex {
        position: clip(x1, y, view_w, view_h),
        uv: [u1, v0],
        style_index: style,
        uv_min,
        uv_max,
        distance_range: range,
    };
    let c = Vertex {
        position: clip(x, y1, view_w, view_h),
        uv: [u0, v1],
        style_index: style,
        uv_min,
        uv_max,
        distance_range: range,
    };
    let d = Vertex {
        position: clip(x1, y1, view_w, view_h),
        uv: [u1, v1],
        style_index: style,
        uv_min,
        uv_max,
        distance_range: range,
    };
    out.extend_from_slice(&[a, c, b, b, c, d]);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalized_glyph_coordinates() {
        assert_eq!(clip(0., 0., 100., 100.), [-1., 1.]);
        assert_eq!(clip(100., 100., 100., 100.), [1., -1.]);
    }
    #[test]
    fn host_vertex_alignment() {
        assert_eq!(std::mem::size_of::<Vertex>(), 40);
    }
}
