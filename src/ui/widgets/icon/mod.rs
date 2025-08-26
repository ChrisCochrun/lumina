// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Lazily-generated SVG icon widget for Iced.

mod named;
use std::ffi::OsStr;
use std::sync::Arc;

pub use named::{IconFallback, Named};

mod handle;
pub use handle::{
    from_path, from_raster_bytes, from_raster_pixels, from_svg_bytes,
    Data, Handle,
};

use derive_setters::Setters;
use iced::advanced::{image, svg};
use iced::widget::{Image, Svg};
use iced::Element;
use iced::Rotation;
use iced::{ContentFit, Length, Rectangle};

/// Create an [`Icon`] from a pre-existing [`Handle`]
pub fn icon(handle: Handle) -> Icon {
    Icon {
        content_fit: ContentFit::Fill,
        handle,
        height: None,
        size: 16,
        rotation: None,
        width: None,
    }
}

/// Create an icon handle from its XDG icon name.
pub fn from_name(name: impl Into<Arc<str>>) -> Named {
    Named::new(name)
}

/// An image which may be an SVG or PNG.
#[must_use]
#[derive(Clone, Setters)]
pub struct Icon {
    #[setters(skip)]
    handle: Handle,
    pub(super) size: u16,
    content_fit: ContentFit,
    #[setters(strip_option)]
    width: Option<Length>,
    #[setters(strip_option)]
    height: Option<Length>,
    #[setters(strip_option)]
    rotation: Option<Rotation>,
}

impl Icon {
    #[must_use]
    pub fn into_svg_handle(
        self,
    ) -> Option<iced::widget::svg::Handle> {
        match self.handle.data {
            Data::Name(named) => {
                if let Some(path) = named.path() {
                    if path
                        .extension()
                        .is_some_and(|ext| ext == OsStr::new("svg"))
                    {
                        return Some(
                            iced::advanced::svg::Handle::from_path(
                                path,
                            ),
                        );
                    }
                }
            }

            Data::Image(_) => (),
            Data::Svg(handle) => return Some(handle),
        }

        None
    }

    #[must_use]
    fn view<'a, Message: 'a>(self) -> Element<'a, Message> {
        let from_image = |handle| {
            Image::new(handle)
                .width(self.width.unwrap_or_else(|| {
                    Length::Fixed(f32::from(self.size))
                }))
                .height(self.height.unwrap_or_else(|| {
                    Length::Fixed(f32::from(self.size))
                }))
                .rotation(self.rotation.unwrap_or_default())
                .content_fit(self.content_fit)
                .into()
        };

        let from_svg = |handle| {
            Svg::<crate::Theme>::new(handle)
                .width(self.width.unwrap_or_else(|| {
                    Length::Fixed(f32::from(self.size))
                }))
                .height(self.height.unwrap_or_else(|| {
                    Length::Fixed(f32::from(self.size))
                }))
                .rotation(self.rotation.unwrap_or_default())
                .content_fit(self.content_fit)
                .into()
        };

        match self.handle.data {
            Data::Name(named) => {
                if let Some(path) = named.path() {
                    if path
                        .extension()
                        .is_some_and(|ext| ext == OsStr::new("svg"))
                    {
                        from_svg(svg::Handle::from_path(path))
                    } else {
                        from_image(image::Handle::from_path(path))
                    }
                } else {
                    let bytes: &'static [u8] = &[];
                    from_svg(svg::Handle::from_memory(bytes))
                }
            }

            Data::Image(handle) => from_image(handle),
            Data::Svg(handle) => from_svg(handle),
        }
    }
}

impl<'a, Message: 'a> From<Icon> for Element<'a, Message> {
    fn from(icon: Icon) -> Self {
        icon.view::<Message>()
    }
}

/// Draw an icon in the given bounds via the runtime's renderer.
pub fn draw(
    renderer: &mut iced::Renderer,
    handle: &Handle,
    icon_bounds: Rectangle,
) {
    enum IcedHandle {
        Svg(svg::Handle),
        Image(image::Handle),
    }

    let iced_handle = match handle.clone().data {
        Data::Name(named) => named.path().map(|path| {
            if path
                .extension()
                .is_some_and(|ext| ext == OsStr::new("svg"))
            {
                IcedHandle::Svg(svg::Handle::from_path(path))
            } else {
                IcedHandle::Image(image::Handle::from_path(path))
            }
        }),

        Data::Image(handle) => Some(IcedHandle::Image(handle)),
        Data::Svg(handle) => Some(IcedHandle::Svg(handle)),
    };

    match iced_handle {
        Some(IcedHandle::Svg(handle)) => svg::Renderer::draw_svg(
            renderer,
            svg::Svg::new(handle),
            icon_bounds,
        ),

        Some(IcedHandle::Image(handle)) => {
            image::Renderer::draw_image(
                renderer,
                (&handle).into(),
                icon_bounds,
            );
        }

        None => {}
    }
}
