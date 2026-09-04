use std::time::{Duration, Instant};

use cosmic::iced::{Point, Rectangle, Size, animation};
use serde::{
    Deserialize, Serialize,
    de::{self, Visitor},
    ser::SerializeStruct,
};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SlideProps {
    opacity: f32,
    translation: Rect,
    scale: f32,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SlideAnimation {
    from_props: SlideProps,
    to_props: SlideProps,
    duration: Duration,
    easing: Easing,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Animation {
    CrossFade {
        duration: Option<Duration>,
        easing: Option<Easing>,
    },
    SlideUp {
        duration: Option<Duration>,
        easing: Option<Easing>,
    },
    SlideLeft {
        duration: Option<Duration>,
        easing: Option<Easing>,
    },
    ScrollUp {
        duration: Option<Duration>,
        easing: Option<Easing>,
    },
}

impl Animation {
    #[must_use]
    pub fn get_animator(&self, instant: Instant) -> cosmic::iced::Animation<bool> {
        const DURATION_DEFAULT: Duration = Duration::from_millis(500);
        const EASING_DEFAULT: Easing = Easing::EaseOut;
        match self {
            Self::CrossFade { duration, easing }
            | Self::SlideUp { duration, easing }
            | Self::SlideLeft { duration, easing }
            | Self::ScrollUp { duration, easing } => {
                let mut animator = cosmic::iced::Animation::new(false);
                if let Some(duration) = duration {
                    animator = animator.duration(*duration);
                } else {
                    animator = animator.duration(DURATION_DEFAULT);
                }
                if let Some(easing) = easing {
                    animator = animator.easing(easing.ease());
                } else {
                    animator = animator.easing(EASING_DEFAULT.ease());
                }
                animator.go(true, instant)
            }
        }
    }
    #[must_use]
    pub fn to_string(&self) -> String {
        match self {
            Self::CrossFade { .. } => "Cross Fade".to_string(),
            Self::SlideUp { .. } => "Slide Up".to_string(),
            Self::SlideLeft { .. } => "Slide Left".to_string(),
            Self::ScrollUp { .. } => "Scrolling Up Text".to_string(),
        }
    }

    #[must_use]
    pub const fn easing(self, new_easing: Easing) -> Self {
        match self {
            Self::CrossFade { duration, .. } => Self::CrossFade {
                duration,
                easing: Some(new_easing),
            },
            Self::SlideUp { duration, .. } => Self::SlideUp {
                duration,
                easing: Some(new_easing),
            },
            Self::SlideLeft { duration, .. } => Self::SlideLeft {
                duration,
                easing: Some(new_easing),
            },
            Self::ScrollUp { duration, .. } => Self::ScrollUp {
                duration,
                easing: Some(new_easing),
            },
        }
    }

    #[must_use]
    pub const fn duration(self, new_duration: Duration) -> Self {
        match self {
            Self::CrossFade { easing, .. } => Self::CrossFade {
                duration: Some(new_duration),
                easing,
            },
            Self::SlideUp { easing, .. } => Self::SlideUp {
                duration: Some(new_duration),
                easing,
            },
            Self::SlideLeft { easing, .. } => Self::SlideLeft {
                duration: Some(new_duration),
                easing,
            },
            Self::ScrollUp { easing, .. } => Self::ScrollUp {
                duration: Some(new_duration),
                easing,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
}

impl Easing {
    #[must_use]
    pub const fn ease(&self) -> animation::Easing {
        match self {
            Self::Linear => animation::Easing::Linear,
            Self::EaseIn => animation::Easing::EaseIn,
            Self::EaseOut => animation::Easing::EaseOut,
            Self::EaseInOut => animation::Easing::EaseInOut,
            Self::EaseInQuad => animation::Easing::EaseInQuad,
            Self::EaseOutQuad => animation::Easing::EaseOutQuad,
            Self::EaseInOutQuad => animation::Easing::EaseInOutQuad,
            Self::EaseInCubic => animation::Easing::EaseInCubic,
            Self::EaseOutCubic => animation::Easing::EaseOutCubic,
            Self::EaseInOutCubic => animation::Easing::EaseInOutCubic,
            Self::EaseInQuart => animation::Easing::EaseInQuart,
            Self::EaseOutQuart => animation::Easing::EaseOutQuart,
            Self::EaseInOutQuart => animation::Easing::EaseInOutQuart,
            Self::EaseInQuint => animation::Easing::EaseInQuint,
            Self::EaseOutQuint => animation::Easing::EaseOutQuint,
            Self::EaseInOutQuint => animation::Easing::EaseInOutQuint,
            Self::EaseInExpo => animation::Easing::EaseInExpo,
            Self::EaseOutExpo => animation::Easing::EaseOutExpo,
            Self::EaseInOutExpo => animation::Easing::EaseInOutExpo,
            Self::EaseInCirc => animation::Easing::EaseInCirc,
            Self::EaseOutCirc => animation::Easing::EaseOutCirc,
            Self::EaseInOutCirc => animation::Easing::EaseInOutCirc,
            Self::EaseInBack => animation::Easing::EaseInBack,
            Self::EaseOutBack => animation::Easing::EaseOutBack,
            Self::EaseInOutBack => animation::Easing::EaseInOutBack,
            Self::EaseInElastic => animation::Easing::EaseInElastic,
            Self::EaseOutElastic => animation::Easing::EaseOutElastic,
            Self::EaseInOutElastic => animation::Easing::EaseInOutElastic,
            Self::EaseInBounce => animation::Easing::EaseInBounce,
            Self::EaseOutBounce => animation::Easing::EaseOutBounce,
            Self::EaseInOutBounce => animation::Easing::EaseInOutBounce,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct Rect(Rectangle);

impl Serialize for Rect {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Rect", 4)?;
        state.serialize_field("x", &self.0.x)?;
        state.serialize_field("y", &self.0.y)?;
        state.serialize_field("width", &self.0.width)?;
        state.serialize_field("height", &self.0.height)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Rect {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            X,
            Y,
            Width,
            Height,
        }

        struct RectVisitor;

        impl<'de> Visitor<'de> for RectVisitor {
            type Value = Rect;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Rect")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Rect, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut x = None;
                let mut y = None;
                let mut width = None;
                let mut height = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::X => {
                            if x.is_some() {
                                return Err(de::Error::duplicate_field("x"));
                            }
                            x = Some(map.next_value()?);
                        }
                        Field::Y => {
                            if y.is_some() {
                                return Err(de::Error::duplicate_field("y"));
                            }
                            y = Some(map.next_value()?);
                        }
                        Field::Width => {
                            if width.is_some() {
                                return Err(de::Error::duplicate_field("width"));
                            }
                            width = Some(map.next_value()?);
                        }
                        Field::Height => {
                            if height.is_some() {
                                return Err(de::Error::duplicate_field("height"));
                            }
                            height = Some(map.next_value()?);
                        }
                    }
                }
                let x = x.ok_or_else(|| de::Error::missing_field("x"))?;
                let y = y.ok_or_else(|| de::Error::missing_field("y"))?;
                let width = width.ok_or_else(|| de::Error::missing_field("width"))?;
                let height = height.ok_or_else(|| de::Error::missing_field("height"))?;
                Ok(Rect(Rectangle::new(
                    Point::new(x, y),
                    Size::new(width, height),
                )))
            }
        }

        const FIELDS: &[&str] = &["x", "y", "width", "height"];
        deserializer.deserialize_struct("Rect", FIELDS, RectVisitor)
    }
}
