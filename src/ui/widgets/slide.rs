use std::time::{Duration, Instant};

use cosmic::iced::{
    Border, Color, ContentFit, Point, Radius, Shadow, Size, core as iced_core,
    widget as iced_widget,
};
use cosmic::widget::Id;
use iced_core::event::Event;
use iced_core::widget::{Operation, Tree};
use iced_core::{
    Clipboard, Element, Layout, Length, Rectangle, Shell, Widget, layout, mouse, renderer,
};
use iced_wgpu::core::renderer::Quad;
use iced_wgpu::primitive::Renderer as PrimitiveRenderer;
use iced_widget::image::Handle;

use crate::core::animation::{Animation, Easing};

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
        + iced_core::image::Renderer<Handle = Handle>,
    <Renderer as iced_core::image::Renderer>::Handle: 'a,
{
    Slide::new(slide, previous_slide, next_slide, video, settings)
}

#[allow(missing_debug_implementations)]
pub struct Slide<'a, Message, Theme, Renderer>
where
    Renderer: PrimitiveRenderer + iced_core::Renderer + iced_core::image::Renderer,
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

#[derive(PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}

impl<'a, Message, Theme, Renderer> Slide<'a, Message, Theme, Renderer>
where
    Renderer: PrimitiveRenderer
        + iced_core::Renderer
        + iced_core::image::Renderer
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
            video: video.map(|video| video.into()),
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

    pub(crate) fn content_fit(mut self, content_fit: ContentFit) -> Self {
        self.content_fit = content_fit;
        self
    }

    pub(crate) fn animate(mut self, animation_state: AnimationState) -> Self {
        self.animation_state = animation_state;
        self
    }

    #[inline(always)]
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
                renderer.draw_image(
                    iced_core::image::Image {
                        handle: allocation.handle().clone(),
                        filter_method: iced_core::image::FilterMethod::Nearest,
                        rotation: iced_core::Radians(0.0),
                        border_radius: Radius::new(0.0),
                        opacity,
                        snap: true,
                    },
                    bounds,
                    clip_bounds,
                )
            });
        } else {
            if let Some(handle) = &background.image_handle {
                let _ = renderer.load_image(handle);
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
                    )
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
        + PrimitiveRenderer
        + cosmic::iced::advanced::image::Renderer<Handle = Handle>,
{
    fn children(&self) -> Vec<Tree> {
        if let Some(video) = &self.video {
            vec![Tree::new(video)]
        } else {
            Vec::new()
        }
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

    #[inline(always)]
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let intrisic_size = self
            .slide
            .background()
            .image_handle
            .as_ref()
            .map(|handle| {
                let _ = renderer.load_image(handle);
                renderer.measure_image(handle)
            })
            .flatten();
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

        if let Some(video) = &self.video {
            video.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor_position,
                viewport,
                renderer,
            )
        } else {
            mouse::Interaction::None
        }
    }

    #[inline(always)]
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
        // let content_layout = layout
        //     .children()
        //     .next()
        //     .expect("There should always be a child");

        let bounds = layout.bounds();
        let clip_bounds = layout
            .bounds()
            .intersection(viewport)
            .unwrap_or(layout.bounds());
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
            )
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

        let (
            mut current_foreground_position,
            mut prev_foreground_position,
            mut next_foreground_position,
        ) = (
            Point::new(bounds.x, bounds.y),
            Point::new(viewport.x - viewport.width, viewport.y - viewport.height),
            Point::new(bounds.x, next_ending_pos),
        );

        if let AnimationState::Running {
            direction,
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

                    let prev_slide_y = if new_slide_progress == &0.0 {
                        bounds.y
                    } else {
                        bounds.y - bounds.height * new_slide_progress
                    };
                    let next_foreground_y = next_ending_pos
                        + (viewport.height - bounds.y)
                        - ((viewport.height - bounds.y) * new_slide_progress);

                    current_foreground_position = Point::new(bounds.x, foreground_y);
                    next_foreground_position = Point::new(bounds.x, next_foreground_y);
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
                    if prev_slide.background() != background {
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
                            )
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
                            1.0,
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
                } else if self.settings.delegate {
                    if let Some(allocation) = &self.slide.thumbnail {
                        renderer.with_layer(bounds, |renderer| {
                            renderer.draw_image(
                                iced_core::image::Image {
                                    handle: allocation.handle().clone(),
                                    filter_method:
                                        iced_core::image::FilterMethod::Nearest,
                                    rotation: iced_core::Radians(0.0),
                                    border_radius: Radius::new(0.0),
                                    opacity: current_slide_opacity,
                                    snap: true,
                                },
                                bounds,
                                clip_bounds,
                            )
                        });
                    }
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
                        )
                    })
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
                )
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
                    Rectangle::new(next_foreground_position, bounds.size()),
                    clip_bounds,
                )
            });
        }
        if let Some(text) = &self.slide.text_svg
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
                        opacity: current_slide_opacity,
                        snap: true,
                    },
                    Rectangle::new(current_foreground_position, bounds.size()),
                    clip_bounds,
                )
            });
        }
    }

    // fn overlay<'b>(
    //     &'b mut self,
    //     tree: &'b mut Tree,
    //     layout: Layout<'b>,
    //     renderer: &Renderer,
    //     viewport: &Rectangle,
    //     translation: Vector,
    // ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
    //     if let Some(video) = &mut self.video {
    //         video.as_widget_mut().overlay(
    //             &mut tree.children[0],
    //             layout
    //                 .children()
    //                 .next()
    //                 .expect("There should always be a child")
    //                 .with_virtual_offset(layout.virtual_offset()),
    //             renderer,
    //             viewport,
    //             translation,
    //         )
    //     } else {
    //         None
    //     }
    // }

    // fn drag_destinations(
    //     &self,
    //     state: &Tree,
    //     layout: Layout<'_>,
    //     renderer: &Renderer,
    //     dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    // ) {
    //     if let Some(video) = &self.video {
    //         let content_layout = layout
    //             .children()
    //             .next()
    //             .expect("There should always be a child");
    //         video.as_widget().drag_destinations(
    //             &state.children[0],
    //             content_layout.with_virtual_offset(layout.virtual_offset()),
    //             renderer,
    //             dnd_rectangles,
    //         );
    //     }
    // }
}

#[allow(clippy::use_self)]
impl<'a, Message, Theme, Renderer> From<Slide<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: 'a
        + iced_core::Renderer
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
