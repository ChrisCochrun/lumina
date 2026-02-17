use cosmic::iced::advanced::layout::{self, Layout};
use cosmic::iced::advanced::renderer;
use cosmic::iced::advanced::widget::{self, Widget};
use cosmic::iced::border;
use cosmic::iced::mouse;
use cosmic::iced::{Color, Element, Length, Rectangle, Size};
use cosmic::iced_wgpu::Primitive;
use cosmic::iced_wgpu::primitive::Renderer as PrimitiveRenderer;

pub struct SlideText {
    _text: String,
    font_size: f32,
}

impl SlideText {
    pub fn new(text: impl AsRef<str>) -> Self {
        let text = text.as_ref();
        Self {
            _text: text.to_string(),
            font_size: 50.0,
        }
    }
}

pub fn slide_text(text: impl AsRef<str>) -> SlideText {
    SlideText::new(text)
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for SlideText
where
    Message: Clone,
    Renderer: PrimitiveRenderer,
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
        renderer
            .draw_primitive(layout.bounds(), TextPrimitive::new());
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

impl<'a, Message, Theme, Renderer> From<SlideText>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + PrimitiveRenderer,
{
    fn from(slide_text: SlideText) -> Self {
        Self::new(slide_text)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TextPrimitive {
    _text_id: u64,
    _size: (u32, u32),
}

impl TextPrimitive {
    pub fn new() -> Self {
        todo!()
    }
}

impl Primitive for TextPrimitive {
    fn prepare(
        &self,
        _device: &cosmic::iced::wgpu::Device,
        _queue: &cosmic::iced::wgpu::Queue,
        _format: cosmic::iced::wgpu::TextureFormat,
        _storage: &mut cosmic::iced_widget::shader::Storage,
        _bounds: &Rectangle,
        _viewport: &cosmic::iced_wgpu::graphics::Viewport,
    ) {
        todo!()
    }

    fn render(
        &self,
        _encoder: &mut cosmic::iced::wgpu::CommandEncoder,
        _storage: &cosmic::iced_widget::shader::Storage,
        _target: &cosmic::iced::wgpu::TextureView,
        _clip_bounds: &Rectangle<u32>,
    ) {
        todo!()
    }
}
