use cosmic::iced::advanced::layout::{self, Layout};
use cosmic::iced::advanced::renderer;
use cosmic::iced::advanced::widget::{self, Widget};
use cosmic::iced::border;
use cosmic::iced::mouse;
use cosmic::iced::{Color, Element, Length, Rectangle, Size};
use femtovg::renderer::WGPURenderer;
use femtovg::{Canvas, TextContext};

pub struct SlideText {
    text: String,
    font_size: f32,
    canvas: Canvas<WGPURenderer>,
}

impl SlideText {
    pub async fn new(text: &str) -> Self {
        let backends = wgpu::Backends::PRIMARY;
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends,
                ..Default::default()
            });
        let surface =
            instance.create_surface(window.clone()).unwrap();
        let adapter = cosmic::iced::wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
            .await
            .expect("Failed to find an appropriate adapter");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: adapter.features(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .expect("failed to device it");
        let renderer = WGPURenderer::new(device, queue);
        let canvas =
            Canvas::new_with_text_context(renderer, text_context)
                .expect("oops femtovg");
        Self {
            text: text.to_owned(),
            font_size: 50.0,
            canvas,
        }
    }
}

fn get_canvas(text_context: TextContext) -> Canvas {
    let renderer = WGPURenderer::new(device, queue);
    Canvas::new_with_text_context(renderer, text_context)
        .expect("oops femtovg")
}

pub fn slide_text(text: &str) -> SlideText {
    SlideText::new(text)
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for SlideText
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(
            self.font_size * 2.0,
            self.font_size * 2.0,
        ))
    }

    fn draw(
        &self,
        _state: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: border::rounded(self.font_size),
                ..renderer::Quad::default()
            },
            Color::BLACK,
        );
    }
}

impl<Message, Theme, Renderer> From<SlideText>
    for Element<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(circle: SlideText) -> Self {
        Self::new(circle)
    }
}
