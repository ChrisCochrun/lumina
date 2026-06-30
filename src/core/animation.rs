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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    pub fn get_animator(&self, instant: Instant) -> cosmic::iced::Animation<bool> {
        const DURATION_DEFAULT: Duration = Duration::from_millis(1500);
        const EASING_DEFAULT: Easing = Easing::EaseOut;
        match self {
            Animation::CrossFade { duration, easing } => {
                let mut animator = cosmic::iced::Animation::new(false);
                if let Some(duration) = duration {
                    animator = animator.duration(duration.clone());
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
            Animation::SlideUp { duration, easing } => {
                let mut animator = cosmic::iced::Animation::new(false);
                if let Some(duration) = duration {
                    animator = animator.duration(duration.clone());
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
            Animation::SlideLeft { duration, easing } => todo!(),
            Animation::ScrollUp { duration, easing } => todo!(),
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Animation::CrossFade { duration, easing } => "Cross Fade".to_string(),
            Animation::SlideUp { duration, easing } => "Slide Up".to_string(),
            Animation::SlideLeft { duration, easing } => todo!(),
            Animation::ScrollUp { duration, easing } => todo!(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
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
    pub fn ease(&self) -> animation::Easing {
        match self {
            Easing::Linear => animation::Easing::Linear,
            Easing::EaseIn => animation::Easing::EaseIn,
            Easing::EaseOut => animation::Easing::EaseOut,
            Easing::EaseInOut => animation::Easing::EaseInOut,
            Easing::EaseInQuad => animation::Easing::EaseInQuad,
            Easing::EaseOutQuad => animation::Easing::EaseOutQuad,
            Easing::EaseInOutQuad => animation::Easing::EaseInOutQuad,
            Easing::EaseInCubic => animation::Easing::EaseInCubic,
            Easing::EaseOutCubic => animation::Easing::EaseOutCubic,
            Easing::EaseInOutCubic => animation::Easing::EaseInOutCubic,
            Easing::EaseInQuart => animation::Easing::EaseInQuart,
            Easing::EaseOutQuart => animation::Easing::EaseOutQuart,
            Easing::EaseInOutQuart => animation::Easing::EaseInOutQuart,
            Easing::EaseInQuint => animation::Easing::EaseInQuint,
            Easing::EaseOutQuint => animation::Easing::EaseOutQuint,
            Easing::EaseInOutQuint => animation::Easing::EaseInOutQuint,
            Easing::EaseInExpo => animation::Easing::EaseInExpo,
            Easing::EaseOutExpo => animation::Easing::EaseOutExpo,
            Easing::EaseInOutExpo => animation::Easing::EaseInOutExpo,
            Easing::EaseInCirc => animation::Easing::EaseInCirc,
            Easing::EaseOutCirc => animation::Easing::EaseOutCirc,
            Easing::EaseInOutCirc => animation::Easing::EaseInOutCirc,
            Easing::EaseInBack => animation::Easing::EaseInBack,
            Easing::EaseOutBack => animation::Easing::EaseOutBack,
            Easing::EaseInOutBack => animation::Easing::EaseInOutBack,
            Easing::EaseInElastic => animation::Easing::EaseInElastic,
            Easing::EaseOutElastic => animation::Easing::EaseOutElastic,
            Easing::EaseInOutElastic => animation::Easing::EaseInOutElastic,
            Easing::EaseInBounce => animation::Easing::EaseInBounce,
            Easing::EaseOutBounce => animation::Easing::EaseOutBounce,
            Easing::EaseInOutBounce => animation::Easing::EaseInOutBounce,
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
