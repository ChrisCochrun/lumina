//! Native iced/libcosmic video timeline: trim in/out + seek position.
//!
//! The parent owns the actual values. The iced widget tree stores only which
//! handle is being dragged. No canvas or GStreamer dependencies.
use cosmic::cosmic_theme::palette;
use cosmic::iced::Padding;
use cosmic::iced::{
    Border, Color, ContentFit, Pixels, Point, Radius, Shadow, Size, advanced::svg,
    border, core as iced_core, touch, widget as iced_widget,
};
use palette::WithAlpha;

use cosmic::widget::Id;
use iced_core::event::Event;
use iced_core::widget::{Operation, Tree};
use iced_core::{
    Clipboard, Element, Layout, Length, Rectangle, Shell, Widget, layout, mouse, renderer,
};
use iced_video_player::Video;
use iced_wgpu::core::renderer::Quad;
use iced_wgpu::core::widget::tree;
use iced_wgpu::primitive::Renderer as PrimitiveRenderer;

const DEFAULT_HEIGHT: f32 = 52.0;
const INSET: f32 = 13.0;
const HANDLE_HIT_RADIUS: f32 = 12.0;

#[derive(Debug, Default)]
struct State {
    dragging: Option<Handle>,
    touch_id: Option<touch::Finger>,
}

#[derive(Debug, Clone, Copy)]
pub enum TimelineEvent {
    TrimStart(f64),
    TrimEnd(f64),
    Seek(f64),
    Released(Handle),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Handle {
    Start,
    End,
    Playhead,
}

pub struct VideoTimeline<'a, Message, Theme>
where
    Theme: Catalog,
{
    id: Id,
    video: Option<&'a Video>,
    width: Length,
    height: f32,
    on_event: Option<fn(TimelineEvent) -> Message>,
    start: f64,
    end: f64,
    class: Theme::Class,
}

impl<'a, Message, Theme> VideoTimeline<'a, Message, Theme>
where
    Theme: Catalog<Class = Class>,
{
    /// Creates an empty [`VideoTimeline`].
    pub(crate) fn new(video: Option<&'a Video>) -> Self {
        VideoTimeline::<Message, Theme> {
            id: Id::unique(),
            video,
            width: Length::Fill,
            height: DEFAULT_HEIGHT,
            start: 0.0,
            end: video.map_or(0.0, |video| video.duration().as_secs_f64()),
            on_event: None,
            class: Theme::Class::default(),
        }
    }

    pub(crate) fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub(crate) fn height(mut self, height: impl Into<Pixels>) -> Self {
        self.height = height.into().0;
        self
    }

    fn time(&self, t: f64) -> f64 {
        if t.is_finite() {
            t.clamp(
                0.0,
                self.video
                    .map_or(0.0, |video| video.duration().as_secs_f64()),
            )
        } else {
            0.0
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for VideoTimeline<'_, Message, Theme>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: iced_core::Renderer
        + iced_core::text::Renderer
        + PrimitiveRenderer
        + iced_core::svg::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> iced_core::Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    #[inline]
    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::None
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::float_cmp)]
    #[inline]
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let Some(video) = self.video else {
            return;
        };
        let bounds = layout.bounds();
        if bounds.intersection(viewport).is_none() || bounds.width <= 2.0 * INSET {
            return;
        }

        let mut quad = |rect: Rectangle, radius: f32, color: Color| {
            if rect.width <= 0.0 || rect.height <= 0.0 {
                return;
            }
            renderer.fill_quad(
                renderer::Quad {
                    bounds: rect,
                    border: border::rounded(radius),
                    ..renderer::Quad::default()
                },
                color,
            );
        };

        let style = theme.style(&self.class);
        let rail = bounds
            .clone()
            .shrink(5.0)
            .shrink(Padding::ZERO.vertical(15.0));
        let rail_radius = rail.height / 4.0;
        quad(rail, rail_radius, style.rail_color);

        let trim_height = rail.height + 10.0;
        let trim_width = 12.0;
        let trim_size = Size::new(trim_width, trim_height);
        let start_x = self.start as f32 * rail.width / video.duration().as_secs_f32();
        let start_position = Point::new(start_x - 3.0 + rail.x, rail.y - 5.0);
        let start_trim_rect = Rectangle::new(start_position, trim_size);
        let end_x = self.end as f32 * rail.width / video.duration().as_secs_f32();
        let end_position = Point::new(end_x - 3.0 + rail.x, rail.y - 5.0);
        let end_trim_rect = Rectangle::new(end_position, trim_size);
        let playhead_location = video.position().as_secs_f64();
        let playhead_x = (playhead_location as f32 * rail.width
            / video.duration().as_secs_f64() as f32)
            + rail.x;
        let played_x = start_x + rail.x;

        let played_rail = Rectangle::new(
            Point::new(played_x, rail.y),
            Size::new((playhead_x - played_x).max(0.0), rail.height),
        );
        quad(played_rail, rail_radius, style.bar_color);

        quad(
            start_trim_rect,
            start_trim_rect.width / 4.0,
            style.trim_handle_colors,
        );
        quad(
            end_trim_rect,
            end_trim_rect.width / 4.0,
            style.trim_handle_colors,
        );

        let playhead_position = Point::new(playhead_x - 1.0, rail.y - 10.0);
        let playhead_bar =
            Rectangle::new(playhead_position, Size::new(2.0, rail.height + 20.0));
        quad(playhead_bar, 0.0, style.playhead_handle_color);

        let playhead_icon: iced_core::svg::Handle = iced_core::svg::Handle::from_memory(
            br##"
<svg xmlns="http://www.w3.org/2000/svg"
     viewBox="0 0 20 20">
  <path
    d="M 4.2 0
       H 15.8
       C 16.6 0, 17.0 0.5, 16.8 1.3
       C 16.2 4.8, 13.9 9.8, 10 16.8
       C 6.1 9.8, 3.8 4.8, 3.2 1.3
       C 3.0 0.5, 3.4 0, 4.2 0
       Z"
    fill="#FF9B38"/>
</svg>
            "##,
        );
        let icon_position =
            Point::new(playhead_position.x - 9.0, playhead_position.y - 5.0);
        let icon_bounds = Rectangle::new(icon_position, Size::new(20.0, 20.0));
        renderer.draw_svg(
            svg::Svg::new(playhead_icon).color(style.playhead_handle_color),
            icon_bounds,
            *viewport,
        );
    }
}

#[allow(clippy::use_self)]
impl<'a, Message, Theme, Renderer> From<VideoTimeline<'a, Message, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + std::clone::Clone,
    Theme: 'a + Catalog,
    Renderer: 'a
        + iced_core::Renderer
        + iced_core::text::Renderer
        + PrimitiveRenderer
        + iced_core::svg::Renderer,
    Theme: 'a,
{
    fn from(
        c: VideoTimeline<'a, Message, Theme>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Self::new(c)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Style {
    /// The rail [`Color`] of the progress indicator.
    pub rail_color: Color,
    /// The bar [`Color`] of the progress indicator.
    pub bar_color: Color,
    /// The border [`Color`] of the progress indicator.
    pub border_color: Option<Color>,
    /// The border radius of the progress indicator.
    pub border_radius: f32,
    /// The color of the trim handles
    pub trim_handle_colors: Color,
    /// The color of the playhead
    pub playhead_handle_color: Color,
}

/// [`Style`] field overrides
#[derive(Clone, Copy, Debug, Default)]
pub struct Class {
    pub rail_color: Option<Color>,
    pub bar_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_radius: Option<f32>,
    pub trim_handle_colors: Option<Color>,
    pub playhead_handle_color: Option<Color>,
}

impl Class {
    pub fn rail_color(mut self, color: impl Into<Color>) -> Self {
        self.rail_color = Some(color.into());
        self
    }

    pub fn bar_color(mut self, color: impl Into<Color>) -> Self {
        self.bar_color = Some(color.into());
        self
    }

    pub fn border_color(mut self, color: impl Into<Color>) -> Self {
        self.border_color = Some(color.into());
        self
    }

    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = Some(radius);
        self
    }

    pub fn trim_handle_colors(mut self, color: impl Into<Color>) -> Self {
        self.trim_handle_colors = Some(color.into());
        self
    }

    pub fn playhead_handle_color(mut self, color: impl Into<Color>) -> Self {
        self.playhead_handle_color = Some(color.into());
        self
    }

    fn resolve(&self, base: Style) -> Style {
        Style {
            rail_color: self.rail_color.unwrap_or(base.rail_color),
            bar_color: self.bar_color.unwrap_or(base.bar_color),
            border_color: self.border_color.or(base.border_color),
            border_radius: self.border_radius.unwrap_or(base.border_radius),
            trim_handle_colors: self
                .trim_handle_colors
                .unwrap_or(base.trim_handle_colors),
            playhead_handle_color: self
                .playhead_handle_color
                .unwrap_or(base.playhead_handle_color),
        }
    }
}

/// A set of rules that dictate the style of an indicator.
pub trait Catalog: Sized {
    /// The supported class of the [`Catalog`].
    type Class: Default;

    /// Produces the active [`Style`] of an indicator.
    fn style(&self, class: &Self::Class) -> Style;
}

impl Catalog for cosmic::iced::Theme {
    type Class = Class;

    fn style(&self, class: &Self::Class) -> Style {
        let palette = self.extended_palette();
        class.resolve(Style {
            rail_color: palette.background.weak.color,
            bar_color: palette.primary.base.color,
            border_color: None,
            border_radius: 0.0,
            trim_handle_colors: palette.secondary.base.color,
            playhead_handle_color: palette.danger.base.color,
        })
    }
}

impl Catalog for cosmic::Theme {
    type Class = Class;

    fn style(&self, class: &Self::Class) -> Style {
        let theme = self.cosmic();

        let (mut rail_color, bar_color) = match (theme.is_dark, theme.is_high_contrast) {
            (true, true) => (
                theme.palette.neutral_6.into(),
                theme.accent_text_color().into(),
            ),
            (true, false) => {
                (theme.palette.neutral_5.into(), theme.accent_color().into())
            }
            (false, true) => (
                theme.palette.neutral_4.into(),
                theme.accent_text_color().into(),
            ),
            (false, false) => {
                (theme.palette.neutral_3.into(), theme.accent_color().into())
            }
        };

        let trim_handle_colors = theme.palette.accent_green.into();
        let playhead_handle_color = theme.palette.accent_red.into();

        class.resolve(Style {
            rail_color,
            bar_color,
            border_color: if theme.is_high_contrast {
                Some(self.current_container().divider.with_alpha(0.5).into())
            } else {
                None
            },
            border_radius: theme.corner_radii.radius_xl[0],
            trim_handle_colors,
            playhead_handle_color,
        })
    }
}
