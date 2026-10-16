use std::cell::RefCell;
use std::fmt::Display;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use cosmic::iced::{
    Border, Color, ContentFit, Point, Radius, Shadow, Size, core as iced_core,
    widget as iced_widget,
};
use cosmic::widget::Id;
use fontdb::Family;
use iced_core::event::Event;
use iced_core::widget::{Operation, Tree};
use iced_core::{
    Clipboard, Element, Layout, Length, Rectangle, Shell, Widget, layout, mouse, renderer,
};
use iced_wgpu::Primitive;
use iced_wgpu::core::renderer::Quad;
use iced_wgpu::primitive::{Pipeline, Renderer as PrimitiveRenderer};
use iced_widget::image::Handle;
use klyff::cosmic_text::{Attrs, FontSystem, Metrics};
use klyff::{
    EncoderContext, GlyphColoring, StyledText, StyledTextBuilder, TextRenderer,
    TextStyle, TextureAtlas, TextureAtlasDescriptor, cosmic_text,
};

use crate::core::animation::Animation;
use crate::core::slide::TextProperties;

pub fn slide<'a, Message: 'static, Theme, Renderer>(
    slide: &'a crate::core::slide::Slide,
    previous_slide: Option<&'a crate::core::slide::Slide>,
    next_slide: Option<&'a crate::core::slide::Slide>,
    video: Option<impl Into<cosmic::iced::Element<'a, Message, Theme, Renderer>>>,
    settings: SlideSettings<'a>,
) -> Slide<'a, Message, Theme, Renderer>
where
    Theme: iced_widget::container::Catalog,
    <Theme as iced_widget::container::Catalog>::Class<'a>:
        From<cosmic::theme::Container<'a>>,
    Renderer: PrimitiveRenderer
        + iced_core::Renderer
        + iced_core::text::Renderer
        + iced_core::image::Renderer<Handle = Handle>,
    <Renderer as iced_core::image::Renderer>::Handle: 'a,
{
    Slide::new(slide, previous_slide, next_slide, video, settings)
}

#[allow(missing_debug_implementations)]
#[allow(dead_code)]
#[allow(clippy::struct_field_names)]
pub struct Slide<'a, Message, Theme, Renderer>
where
    Renderer: PrimitiveRenderer
        + iced_core::Renderer
        + iced_core::image::Renderer
        + iced_core::text::Renderer,
{
    id: Id,
    slide: &'a crate::core::slide::Slide,
    previous_slide: Option<&'a crate::core::slide::Slide>,
    next_slide: Option<&'a crate::core::slide::Slide>,
    video: Option<cosmic::iced::Element<'a, Message, Theme, Renderer>>,
    settings: SlideSettings<'a>,

    width: Length,
    height: Length,
    content_fit: ContentFit,
    animation_state: AnimationState,
}

pub struct SlideSettings<'a> {
    pub delegate: bool,
    pub hide_mouse: bool,
    pub animation: Option<&'a crate::core::animation::Animation>,
    pub now: Instant,
    pub font_system: Arc<Mutex<FontSystem>>,
}

#[derive(PartialEq)]
pub enum AnimationState {
    Idle,
    Running {
        direction: Direction,
        new_slide_progress: f32,
        prev_slide_progress: f32,
    },
    Done,
}

#[derive(PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}

#[allow(dead_code)]
impl<'a, Message, Theme, Renderer> Slide<'a, Message, Theme, Renderer>
where
    Renderer: PrimitiveRenderer
        + iced_core::Renderer
        + iced_core::image::Renderer
        + iced_core::text::Renderer
        + cosmic::iced::advanced::image::Renderer<Handle = Handle>,
    <Renderer as iced_core::image::Renderer>::Handle: 'a,
{
    /// Creates an empty [`Slide`].
    pub(crate) fn new(
        slide: &'a crate::core::slide::Slide,
        previous_slide: Option<&'a crate::core::slide::Slide>,
        next_slide: Option<&'a crate::core::slide::Slide>,
        video: Option<impl Into<cosmic::iced::Element<'a, Message, Theme, Renderer>>>,
        settings: SlideSettings<'a>,
    ) -> Self {
        Slide {
            id: Id::unique(),
            slide,
            previous_slide,
            next_slide,
            video: video.map(std::convert::Into::into),
            settings,
            width: Length::Fill,
            height: Length::Fill,
            content_fit: ContentFit::Fill,
            animation_state: AnimationState::Idle,
        }
    }

    pub(crate) fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub(crate) fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub(crate) const fn content_fit(mut self, content_fit: ContentFit) -> Self {
        self.content_fit = content_fit;
        self
    }

    pub(crate) const fn animate(mut self, animation_state: AnimationState) -> Self {
        self.animation_state = animation_state;
        self
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::cast_precision_loss)]
    #[inline]
    fn draw_background(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _renderer_style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor_position: mouse::Cursor,
        _viewport: &Rectangle,
        bounds: Rectangle,
        clip_bounds: Rectangle,
        opacity: f32,
    ) {
        let background = self.slide.background();
        if let Some(allocation) = background.image_allocation.as_ref() {
            renderer.with_layer(bounds, |renderer| {
                let size = allocation.size();
                let width = size.width as f32;
                let height = size.height as f32;
                let width_scale = width / bounds.size().width;
                let height_scale = height / bounds.size().height;
                let new_size = Size::new(
                    width
                        / if width_scale > height_scale {
                            width_scale
                        } else {
                            height_scale
                        },
                    height
                        / if width_scale > height_scale {
                            width_scale
                        } else {
                            height_scale
                        },
                );
                let center_x = bounds.x + bounds.width / 2.0;
                let new_x = center_x - new_size.width / 2.0;
                let center_y = bounds.y + bounds.height / 2.0;
                let new_y = center_y - new_size.height / 2.0;
                let alloc_bounds = Rectangle::new(Point { x: new_x, y: new_y }, new_size);

                renderer.draw_image(
                    iced_core::image::Image {
                        handle: allocation.handle().clone(),
                        filter_method: iced_core::image::FilterMethod::Nearest,
                        rotation: iced_core::Radians(0.0),
                        border_radius: Radius::new(0.0),
                        opacity,
                        snap: true,
                    },
                    alloc_bounds,
                    clip_bounds,
                );
            });
        } else if let Some(handle) = &background.image_handle {
            if let Ok(allocation) = renderer.load_image(handle) {
                renderer.with_layer(bounds, |renderer| {
                    let size = allocation.size();
                    let width = size.width as f32;
                    let height = size.height as f32;
                    let width_scale = width / bounds.size().width;
                    let height_scale = height / bounds.size().height;
                    let new_size = Size::new(
                        width
                            / if width_scale > height_scale {
                                width_scale
                            } else {
                                height_scale
                            },
                        height
                            / if width_scale > height_scale {
                                width_scale
                            } else {
                                height_scale
                            },
                    );
                    let center_x = bounds.x + bounds.width / 2.0;
                    let new_x = center_x - new_size.width / 2.0;
                    let center_y = bounds.y + bounds.height / 2.0;
                    let new_y = center_y - new_size.height / 2.0;
                    let alloc_bounds =
                        Rectangle::new(Point { x: new_x, y: new_y }, new_size);

                    renderer.draw_image(
                        iced_core::image::Image {
                            handle: allocation.handle().clone(),
                            filter_method: iced_core::image::FilterMethod::Nearest,
                            rotation: iced_core::Radians(0.0),
                            border_radius: Radius::new(0.0),
                            opacity,
                            snap: true,
                        },
                        alloc_bounds,
                        clip_bounds,
                    );
                });
            } else {
                renderer.with_layer(bounds, |renderer| {
                    renderer.draw_image(
                        iced_core::image::Image {
                            handle: handle.clone(),
                            filter_method: iced_core::image::FilterMethod::Nearest,
                            rotation: iced_core::Radians(0.0),
                            border_radius: Radius::new(0.0),
                            opacity,
                            snap: true,
                        },
                        bounds,
                        clip_bounds,
                    );
                });
            }
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Slide<'_, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer
        + iced_core::image::Renderer
        + iced_core::text::Renderer
        + PrimitiveRenderer
        + cosmic::iced::advanced::image::Renderer<Handle = Handle>,
{
    fn children(&self) -> Vec<Tree> {
        self.video
            .as_ref()
            .map_or_else(Vec::new, |video| vec![Tree::new(video)])
    }

    fn diff(&mut self, tree: &mut Tree) {
        if let Some(video) = &mut self.video {
            tree.diff_children(std::slice::from_mut(video));
        }
    }

    fn size(&self) -> iced_core::Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    #[allow(clippy::cast_precision_loss)]
    #[inline]
    fn layout(
        &mut self,
        _tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let intrisic_size =
            self.slide
                .background()
                .image_handle
                .as_ref()
                .and_then(|handle| {
                    let _ = renderer.load_image(handle);
                    renderer.measure_image(handle)
                });
        let original_size = self.size();
        let measured_size = intrisic_size.map_or_else(
            || Size::new(1920.0, 1080.0),
            |size| Size::new(size.width as f32, size.height as f32),
        );
        layout::Node::new(
            limits
                .resolve(original_size.width, original_size.height, measured_size)
                .ratio(16.0 / 9.0),
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            if let Some(video) = &mut self.video {
                video.as_widget_mut().operate(
                    &mut tree.children[0],
                    layout,
                    renderer,
                    operation,
                );
            }
        });
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
        if let Some(video) = &mut self.video {
            video.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor_position,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        // let content_layout = layout
        //     .children()
        //     .next()
        //     .expect("There should always be a child");

        self.video
            .as_ref()
            .map_or(mouse::Interaction::None, |video| {
                video.as_widget().mouse_interaction(
                    &tree.children[0],
                    layout,
                    cursor_position,
                    viewport,
                    renderer,
                )
            })
    }

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
        let bounds = layout.bounds();

        // let content_layout = layout
        //     .children()
        //     .next()
        //     .expect("There should always be a child");

        let clip_bounds = layout
            .bounds()
            .intersection(viewport)
            .unwrap_or_else(|| layout.bounds());
        let background = self.slide.background();

        if self.video.is_none() {
            renderer.fill_quad(
                Quad {
                    bounds,
                    border: Border {
                        color: Color::BLACK,
                        width: 0.0,
                        radius: Radius::new(0.0),
                    },
                    shadow: Shadow::default(),
                    snap: true,
                },
                cosmic::iced::Background::Color(Color::BLACK),
            );
        }
        let (mut current_slide_opacity, mut prev_slide_opacity, mut next_text_opacity) = (
            1.0,
            0.0,
            if matches!(self.settings.animation, Some(Animation::ScrollUp { .. })) {
                0.5
            } else {
                1.0
            },
        );

        let next_ending_pos = bounds.y + bounds.height / 2.0;

        let mut next_foreground_size =
            if matches!(self.settings.animation, Some(Animation::ScrollUp { .. })) {
                // debug!("***********");
                bounds.size() * 0.8
            } else {
                bounds.size()
            };

        let next_size_difference = bounds.size().width - next_foreground_size.width;
        let next_foreground_x = next_size_difference / 2.0 + bounds.x;
        let mut next_foreground_position = Point::new(next_foreground_x, next_ending_pos);

        let (
            mut current_foreground_position,
            mut current_foreground_size,
            mut prev_foreground_position,
        ) = (
            Point::new(bounds.x, bounds.y),
            bounds.size(),
            Point::new(viewport.x - viewport.width, viewport.y - viewport.height),
        );

        if let AnimationState::Running {
            direction: _,
            new_slide_progress,
            prev_slide_progress,
        } = &self.animation_state
        {
            match self.settings.animation {
                Some(Animation::CrossFade { .. }) => {
                    current_slide_opacity = *new_slide_progress;
                    prev_slide_opacity = *prev_slide_progress;
                }
                Some(Animation::SlideUp { .. }) => {
                    prev_slide_opacity = 1.0;

                    let foreground_y = bounds.y + (viewport.height - bounds.y)
                        - ((viewport.height - bounds.y) * new_slide_progress);
                    let prev_slide_y = if new_slide_progress == &0.0 {
                        bounds.y
                    } else {
                        bounds.y - bounds.height * new_slide_progress
                    };

                    current_foreground_position = Point::new(bounds.x, foreground_y);
                    prev_foreground_position = Point::new(bounds.x, prev_slide_y);
                }
                Some(Animation::ScrollUp { .. }) => {
                    prev_slide_opacity = 1.0;
                    current_slide_opacity = new_slide_progress.max(0.65);
                    next_text_opacity = 0.65_f32;

                    let foreground_y = bounds.y + (next_ending_pos - bounds.y)
                        - (next_ending_pos - bounds.y) * new_slide_progress;

                    next_foreground_size = bounds.size() * 0.8;

                    let _size_progress = bounds.size() * *new_slide_progress;
                    current_foreground_size = next_foreground_size
                        + (bounds.size() - next_foreground_size) * *new_slide_progress;

                    let current_size_difference =
                        bounds.size().width - current_foreground_size.width;
                    let current_foreground_x = current_size_difference / 2.0 + bounds.x;

                    let _og_size = bounds.size();

                    // debug!(?next_foreground_size, ?og_size, ?size_progress);

                    let prev_slide_y = if new_slide_progress == &0.0 {
                        bounds.y
                    } else {
                        bounds.y - bounds.height * new_slide_progress
                    };
                    let next_foreground_y = next_ending_pos
                        + (viewport.height - bounds.y)
                        - ((viewport.height - bounds.y) * new_slide_progress);

                    current_foreground_position =
                        Point::new(current_foreground_x, foreground_y);
                    next_foreground_position =
                        Point::new(next_foreground_x, next_foreground_y);
                    prev_foreground_position = Point::new(bounds.x, prev_slide_y);
                }
                Some(Animation::SlideLeft { .. }) => todo!(),
                None => (),
            }
        }

        match background.kind {
            crate::core::slide::BackgroundKind::Image => {
                if let Some(prev_slide) = self.previous_slide
                    && let Some(prev_allocation) =
                        prev_slide.background().image_allocation.as_ref()
                {
                    if prev_slide.background() == background {
                        self.draw_background(
                            tree,
                            renderer,
                            theme,
                            renderer_style,
                            layout,
                            cursor_position,
                            viewport,
                            bounds,
                            clip_bounds,
                            1.0,
                        );
                    } else {
                        renderer.with_layer(bounds, |renderer| {
                            renderer.draw_image(
                                iced_core::image::Image {
                                    handle: prev_allocation.handle().clone(),
                                    filter_method:
                                        iced_core::image::FilterMethod::Nearest,
                                    rotation: iced_core::Radians(0.0),
                                    border_radius: Radius::new(0.0),
                                    opacity: prev_slide_opacity,
                                    snap: true,
                                },
                                bounds,
                                clip_bounds,
                            );
                        });
                        self.draw_background(
                            tree,
                            renderer,
                            theme,
                            renderer_style,
                            layout,
                            cursor_position,
                            viewport,
                            bounds,
                            clip_bounds,
                            current_slide_opacity,
                        );
                    }
                } else {
                    self.draw_background(
                        tree,
                        renderer,
                        theme,
                        renderer_style,
                        layout,
                        cursor_position,
                        viewport,
                        bounds,
                        clip_bounds,
                        current_slide_opacity,
                    );
                }
            }
            crate::core::slide::BackgroundKind::Video => {
                if let Some(video) = &self.video
                    && !self.settings.delegate
                {
                    video.as_widget().draw(
                        &tree.children[0],
                        renderer,
                        theme,
                        renderer_style,
                        layout,
                        cursor_position,
                        viewport,
                    );
                } else if self.settings.delegate
                    && let Some(allocation) = &self.slide.thumbnail
                {
                    renderer.with_layer(bounds, |renderer| {
                        renderer.draw_image(
                            iced_core::image::Image {
                                handle: allocation.handle().clone(),
                                filter_method: iced_core::image::FilterMethod::Nearest,
                                rotation: iced_core::Radians(0.0),
                                border_radius: Radius::new(0.0),
                                opacity: current_slide_opacity,
                                snap: true,
                            },
                            bounds,
                            clip_bounds,
                        );
                    });
                }
            }
            crate::core::slide::BackgroundKind::Pdf => {
                if let Some(pdf) = &self.slide.pdf_page() {
                    let _ = renderer.load_image(pdf);
                    renderer.with_layer(bounds, |renderer| {
                        renderer.draw_image(
                            iced_core::image::Image {
                                handle: pdf.clone(),
                                filter_method: iced_core::image::FilterMethod::Nearest,
                                rotation: iced_core::Radians(0.0),
                                border_radius: Radius::new(0.0),
                                opacity: current_slide_opacity,
                                snap: true,
                            },
                            bounds,
                            clip_bounds,
                        );
                    });
                }
            }
            crate::core::slide::BackgroundKind::Html => todo!(),
        }
        if let Some(slide) = &self.previous_slide
            && let Some(text) = &slide.text_svg
            && let Some(handle) = &text.handle
        {
            let _ = renderer.load_image(handle);
            renderer.with_layer(bounds, |renderer| {
                renderer.draw_image(
                    iced_core::image::Image {
                        handle: handle.clone(),
                        filter_method: iced_core::image::FilterMethod::Nearest,
                        rotation: iced_core::Radians(0.0),
                        border_radius: Radius::new(0.0),
                        opacity: prev_slide_opacity,
                        snap: true,
                    },
                    Rectangle::new(prev_foreground_position, bounds.size()),
                    clip_bounds,
                );
            });
        }
        if let Some(slide) = &self.next_slide
            && let Some(text) = &slide.text_svg
            && let Some(handle) = &text.handle
        {
            let _ = renderer.load_image(handle);
            renderer.with_layer(bounds, |renderer| {
                renderer.draw_image(
                    iced_core::image::Image {
                        handle: handle.clone(),
                        filter_method: iced_core::image::FilterMethod::Nearest,
                        rotation: iced_core::Radians(0.0),
                        border_radius: Radius::new(0.0),
                        opacity: next_text_opacity,
                        snap: true,
                    },
                    Rectangle::new(next_foreground_position, next_foreground_size),
                    clip_bounds,
                );
            });
        }

        if self.slide.text.len() > 0 {
            let text = &self.slide.text;
            let mut text_renderer =
                TextPrimitive::new(Arc::clone(&self.settings.font_system));
            let text_props = self.slide.text_properties.clone().unwrap_or_default();
            text_renderer.build_text(&self.slide.text, &text_props, &bounds);

            renderer.with_layer(bounds, |renderer| {
                renderer.draw_primitive(
                    Rectangle::new(current_foreground_position, current_foreground_size),
                    text_renderer,
                );
            });
            // tracing::warn!(text, "SLIDE DRAW: bounds={bounds:?}, viewport={viewport:?}");
        }
        // if let Some(text) = &self.slide.text_svg
        //     && let Some(handle) = &text.handle
        // {
        //     let _ = renderer.load_image(handle);
        //     renderer.with_layer(bounds, |renderer| {
        //         renderer.draw_image(
        //             iced_core::image::Image {
        //                 handle: handle.clone(),
        //                 filter_method: iced_core::image::FilterMethod::Nearest,
        //                 rotation: iced_core::Radians(0.0),
        //                 border_radius: Radius::new(0.0),
        //                 opacity: current_slide_opacity,
        //                 snap: true,
        //             },
        //             Rectangle::new(current_foreground_position, current_foreground_size),
        //             clip_bounds,
        //         );
        //     });
        // }
    }
}

#[allow(clippy::use_self)]
impl<'a, Message, Theme, Renderer> From<Slide<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: 'a
        + iced_core::Renderer
        + iced_core::text::Renderer
        + PrimitiveRenderer
        + iced_core::image::Renderer<Handle = cosmic::widget::image::Handle>,
    Theme: 'a,
{
    fn from(
        c: Slide<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Self::new(c)
    }
}

pub(crate) struct TextPrimitive {
    font_system: Arc<Mutex<FontSystem>>,
    texts: Mutex<Vec<StyledTextBuilder>>,
    text_renderer: Mutex<Option<TextRenderer>>,
}

impl std::fmt::Debug for TextPrimitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Ok(texts) = self.texts.lock() else {
            return Err(std::fmt::Error::default());
        };
        write!(f, "TextPrimitive: with {}", texts.len())
    }
}

pub struct TextPipeline {
    format: iced_wgpu::wgpu::TextureFormat,
    atlas: TextureAtlas,
}

impl Pipeline for TextPipeline {
    fn new(
        device: &iced_wgpu::wgpu::Device,
        queue: &iced_wgpu::wgpu::Queue,
        format: iced_wgpu::wgpu::TextureFormat,
    ) -> Self
    where
        Self: Sized,
    {
        tracing::error!("{:?}", device.clone().limits());
        Self {
            format,
            atlas: TextureAtlas::new(&device, TextureAtlasDescriptor::default()),
        }
    }
}

impl TextPrimitive {
    pub fn new(font_system: Arc<Mutex<FontSystem>>) -> Self {
        Self {
            font_system,
            texts: Mutex::new(Vec::new()),
            text_renderer: Mutex::new(None),
        }
    }

    pub fn push_text(&mut self, text: StyledTextBuilder) {
        let Ok(mut texts) = self.texts.lock() else {
            return;
        };
        texts.push(text)
    }

    pub fn build_text(
        &mut self,
        text: impl AsRef<str>,
        text_props: &TextProperties,
        bounds: &Rectangle,
    ) {
        let styled_text = if let Ok(mut font_system) = self.font_system.lock() {
            tracing::warn!("BUILD TEXT: bounds={bounds:?}, text={:?}", text.as_ref(),);
            // tracing::warn!("{text_props:?}");
            let metrics =
                Metrics::relative(text_props.font_size.unwrap_or(100.0) * 1.5, 1.2);
            let mut attrs = Attrs::new()
                .family(
                    text_props
                        .font
                        .as_ref()
                        .map_or(Family::SansSerif, |font| Family::Name(font)),
                )
                .metrics(metrics);
            let default_attrs = attrs.clone();
            let mut text_builder = StyledTextBuilder::new(
                klyff::Rect::from_xywh(0.0, 0.0, bounds.width, bounds.height),
                &mut font_system,
                metrics.clone(),
                &default_attrs,
            );
            if let Some(style) = text_props.font_style {
                let style = match style {
                    iced_core::font::Style::Normal => fontdb::Style::Normal,
                    iced_core::font::Style::Italic => fontdb::Style::Italic,
                    iced_core::font::Style::Oblique => fontdb::Style::Oblique,
                };
                attrs = attrs.style(style)
            }

            if let Some(weight) = text_props.font_weight {
                let weight = match weight {
                    iced_core::font::Weight::Thin => fontdb::Weight::THIN,
                    iced_core::font::Weight::ExtraLight => fontdb::Weight::EXTRA_LIGHT,
                    iced_core::font::Weight::Light => fontdb::Weight::LIGHT,
                    iced_core::font::Weight::Normal => fontdb::Weight::NORMAL,
                    iced_core::font::Weight::Medium => fontdb::Weight::MEDIUM,
                    iced_core::font::Weight::Semibold => fontdb::Weight::SEMIBOLD,
                    iced_core::font::Weight::Bold => fontdb::Weight::BOLD,
                    iced_core::font::Weight::ExtraBold => fontdb::Weight::EXTRA_BOLD,
                    iced_core::font::Weight::Black => fontdb::Weight::BLACK,
                };
                attrs = attrs.weight(weight)
            }

            if let Some(stretch) = text_props.font_stretch {
                let stretch = match stretch {
                    iced_core::font::Stretch::UltraCondensed => {
                        fontdb::Stretch::UltraCondensed
                    }
                    iced_core::font::Stretch::ExtraCondensed => {
                        fontdb::Stretch::ExtraCondensed
                    }
                    iced_core::font::Stretch::Condensed => fontdb::Stretch::Condensed,
                    iced_core::font::Stretch::SemiCondensed => {
                        fontdb::Stretch::SemiCondensed
                    }
                    iced_core::font::Stretch::Normal => fontdb::Stretch::Normal,
                    iced_core::font::Stretch::SemiExpanded => {
                        fontdb::Stretch::SemiExpanded
                    }
                    iced_core::font::Stretch::Expanded => fontdb::Stretch::Expanded,
                    iced_core::font::Stretch::ExtraExpanded => {
                        fontdb::Stretch::ExtraExpanded
                    }
                    iced_core::font::Stretch::UltraExpanded => {
                        fontdb::Stretch::UltraExpanded
                    }
                };
                attrs = attrs.stretch(stretch)
            }

            let color = text_props.text_color.unwrap_or_default();
            let color = klyff::Color::from_rgb(color.red, color.green, color.blue);
            let stroke_color = text_props.stroke_color.unwrap_or_default();
            let stroke_color = klyff::Color::from_rgb(
                stroke_color.red,
                stroke_color.green,
                stroke_color.blue,
            );
            let stroke_width = text_props.stroke_size.unwrap_or_default();
            let shadow_color = text_props.shadow_color.unwrap_or_default();
            let shadow_color = klyff::GlyphColoring::Solid(klyff::Color::from_rgb(
                shadow_color.red,
                shadow_color.green,
                shadow_color.blue,
            ));
            let shadow_direction = text_props.shadow_offset.unwrap_or_default();
            let shadow_spread = text_props.shadow_size.unwrap_or_default();

            let style = TextStyle {
                color: GlyphColoring::Solid(color),
                stroke_out: klyff::Outline {
                    color: GlyphColoring::Solid(stroke_color),
                    width: stroke_width,
                    roundness: 1.0,
                },
                stroke_in: klyff::Outline {
                    color: GlyphColoring::Solid(klyff::Color::TRANSPARENT),
                    width: 0.0,
                    roundness: 1.0,
                },
                glow_out: klyff::Glow::default(),
                glow_in: klyff::Glow::default(),
                shadow: klyff::Shadow {
                    color: shadow_color,
                    direction: klyff::glam::Vec2::new(
                        shadow_direction.0,
                        shadow_direction.1,
                    ),
                    additional_width: 0.0,
                    spread: shadow_spread,
                    roundness: 1.0,
                },
            };
            text_builder.set_align(Some(text_props.text_alignment.into()));
            text_builder.push_text(text.as_ref(), &attrs, style);
            // tracing::warn!("RUNNING");
            Some(text_builder)
        } else {
            None
        };

        if let Some(styled_text) = styled_text {
            // tracing::debug!("{styled_text:?}");
            self.push_text(styled_text);
        }
    }
}

impl Primitive for TextPrimitive {
    type Pipeline = TextPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &iced_wgpu::wgpu::Device,
        queue: &iced_wgpu::wgpu::Queue,
        bounds: &Rectangle,
        viewport: &iced_wgpu::graphics::Viewport,
    ) {
        let Ok(mut renderer_mutex) = self.text_renderer.lock() else {
            return;
        };
        let renderer = renderer_mutex.get_or_insert_with(|| {
            let features = klyff::Features::all();
            TextRenderer::with_styling(features, device, pipeline.format)
        });

        let new = Arc::clone(&self.font_system);
        let Ok(mut font_system) = new.lock() else {
            tracing::error!("Couldn't use font system");
            return;
        };
        tracing::warn!(
            "TEXT PREPARE: bounds={bounds:?}, viewport physical={:?}, logical={:?}, scale={}",
            viewport.physical_size(),
            viewport.logical_size(),
            viewport.scale_factor(),
        );
        let mut encoder =
            device.create_command_encoder(
                &iced_wgpu::wgpu::CommandEncoderDescriptor::default(),
            );

        let scale = bounds.size().width / viewport.logical_size().width;
        let builders = {
            let Ok(mut texts) = self.texts.lock() else {
                return;
            };
            std::mem::take(&mut *texts)
        };

        let texts = builders
            .into_iter()
            .map(|mut text| {
                text.set_scale(scale as f32);
                text.finish(&mut font_system)
            })
            .collect::<Vec<StyledText>>();
        tracing::debug!("{texts:?}");
        renderer.prepare(
            EncoderContext {
                atlas: &mut pipeline.atlas,
                device,
                queue,
                cmd_encoder: &mut encoder,
                font_system: &mut font_system,
            },
            (bounds.width as u32, bounds.height as u32),
            &texts,
        );
        queue.submit(std::iter::once(encoder.finish()));
    }

    fn draw(
        &self,
        pipeline: &Self::Pipeline,
        render_pass: &mut iced_wgpu::wgpu::RenderPass<'_>,
    ) -> bool {
        // tracing::warn!("{render_pass:?}");
        let Ok(renderer) = self.text_renderer.lock() else {
            return false;
        };
        if let Some(renderer) = renderer.as_ref() {
            renderer.render(render_pass, &pipeline.atlas);
            true
        } else {
            false
        }
    }

    fn render(
        &self,
        pipeline: &Self::Pipeline,
        encoder: &mut iced_wgpu::wgpu::CommandEncoder,
        target: &iced_wgpu::wgpu::TextureView,
        _bounds: &Rectangle<u32>,
    ) {
    }
}
