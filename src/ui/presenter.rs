use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use cosmic::cosmic_theme::Spacing;
use cosmic::iced::alignment::Horizontal;
use cosmic::iced::core::text::{Alignment, Ellipsize, EllipsizeHeightLimit};
use cosmic::iced::font::{Family, Stretch, Style, Weight};
use cosmic::iced::widget::scrollable::{Direction, Scrollbar};
use cosmic::iced::widget::stack;
use cosmic::iced::{
    Animation, Background, Border, Color, ContentFit, Font, Length, Point, Shadow, Vector,
};
use cosmic::prelude::*;
use cosmic::widget::divider::{self, vertical};
use cosmic::widget::{
    Container, Id, JustifyContent, Row, Space, column, container, flex_row,
    image as cosmic_image, menu, mouse_area, popover, responsive, scrollable, space,
    text,
};
use cosmic::{Task, theme};
use derive_more::Debug;
use iced_video_player::{Position, Video, VideoPlayer, gst_pbutils};
use obws::Client;
use obws::responses::scenes::Scene;
use rodio::{Decoder, MixerDeviceSink, Player, Source};
use tracing::{debug, error, info, warn};
use url::Url;

use crate::core::service_items::ServiceItem;
use crate::core::slide::Slide;
use crate::core::slide_actions::{self, ObsAction};
use crate::ui::gst_video::{self, VideoSettings};
use crate::ui::image_loader::ImageLoader;
use crate::ui::scroll_operations::{self, focus_target};
use crate::ui::widgets::loaded_image::loaded_image;
use crate::ui::widgets::slide::AnimationState;
use crate::ui::widgets::{self, slide::slide};
use crate::{BackgroundKind, ViewMode};

// const REFERENCE_WIDTH: f32 = 1920.0;
static DEFAULT_SLIDE: LazyLock<Slide> = LazyLock::new(Slide::default);

// #[derive(Default, Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct Presenter {
    pub service: Arc<Vec<ServiceItem>>,
    pub current_slide: Slide,
    pub activating_slide: Option<Slide>,
    pub old_slide: Option<Slide>,
    pub current_item_index: usize,
    pub current_slide_index: usize,
    pub total_slides: usize,
    pub presentation_video: Option<Video>,
    pub preview_video: Option<Video>,
    pub video_position: f64,
    pub audio: Option<PathBuf>,
    audio_duration: Option<Duration>,
    audio_position: Option<Duration>,
    sink: (Arc<MixerDeviceSink>, Arc<Player>),
    hovered_slide: Option<(usize, usize)>,
    hovered_point: Option<Point>,
    direction_of_slide_change: scroll_operations::Direction,
    scroll_id: Id,
    active_slide_id: Id,
    current_font: Font,
    slide_action_map: Option<HashMap<(usize, usize), Vec<slide_actions::Action>>>,
    obs_client: Option<Arc<Client>>,
    context_menu_id: Option<(usize, usize)>,
    context_point: Point,
    obs_scenes: Option<Vec<Scene>>,
    preview_size: f32,
    pub image_loader: ImageLoader,
    animation: Option<crate::core::animation::Animation>,
    animator: Option<Animation<bool>>,
    now: Instant,
    pub view_mode: ViewMode,
}

#[allow(dead_code)]
pub(crate) enum Action {
    Task(Task<Message>),
    NextSlide,
    PrevSlide,
    ChangeSlide(usize, usize),
    None,
}

#[allow(clippy::large_enum_variant)]
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) enum Message {
    NextSlide,
    PrevSlide,
    ActivateSlide(usize, usize),
    EndVideo,
    StartVideo,
    VideoPos(f64),
    VideoFrame,
    LoadedService,
    MissingPlugin(iced_video_player::gst::Message),
    HoveredSlide(Option<(usize, usize, Point)>),
    ChangeFont(String),
    Error(String),
    None,
    RightClickSlide(usize, usize),
    AssignObsScene(usize),
    UpdateObsScenes(Vec<Scene>),
    #[debug("AddObsClient")]
    AddObsClient(Arc<Client>),
    AssignSlideAction(slide_actions::Action),
    PlayPauseVideo,
    CloseContextMenu,
    ChangePreviewSize(f64),
    Tick(Instant),
}

impl Presenter {
    #[allow(clippy::too_many_lines)]
    pub fn with_items(items: Arc<Vec<ServiceItem>>) -> Self {
        let preview_video = {
            items.first().and_then(|item| {
                item.slides.first().and_then(|slide| {
                    let path = slide.background().path.clone();
                    if !path.exists() {
                        return None;
                    }

                    let url = Url::from_file_path(path).expect(
                        "There should be a video file here",
                    );

                    let video_settings = VideoSettings {
                        mute: true,
                        framerate: 15,
                        appsink_name: "lumina_preview".to_string(),
                    };
                    let result = gst_video::create_video(&url, &video_settings);
                    match result {
                        Ok(mut v) => {
                            v.set_paused(true);
                            Some(v)
                        }
                        Err(e) => {
                            error!(
                                "Had an error creating the video object: {e}, likely the first slide isn't a video"
                            );
                            None
                        }
                    }
                })
            })
        };
        let presentation_video = {
            items.first().and_then(|item| {
                item.slides.first().and_then(|slide| {
                    let path = slide.background().path.clone();
                    if !path.exists() {
                        return None;
                    }

                    let url = Url::from_file_path(path).expect(
                        "There should be a video file here",
                    );

                    let video_settings = VideoSettings {
                        mute: false,
                        framerate: 60,
                        appsink_name: "lumina_video".to_string(),

                    };

                    let result = gst_video::create_video(&url, &video_settings);
                    match result {
                        Ok(mut v) => {
                            v.set_paused(true);
                            Some(v)
                        }
                        Err(e) => {
                            error!(
                                "Had an error creating the video object: {e}, likely the first slide isn't a video"
                            );
                            None
                        }
                    }
                })
            })
        };
        let total_slides: usize = items.iter().fold(0, |a, item| a + item.slides.len());

        let slide = items.first().and_then(|item| item.slides.first());

        let audio = items
            .first()
            .and_then(|item| {
                item.slides
                    .first()
                    .map(super::super::core::slide::Slide::audio)
            })
            .flatten();

        Self {
            current_slide: slide.unwrap_or(&DEFAULT_SLIDE).clone(),
            activating_slide: None,
            old_slide: None,
            current_item_index: 0,
            current_slide_index: 0,
            total_slides,
            preview_video,
            presentation_video,
            audio,
            audio_duration: None,
            audio_position: None,
            service: items,
            video_position: 0.0,
            hovered_slide: None,
            hovered_point: None,
            sink: {
                let stream_handle = rodio::DeviceSinkBuilder::open_default_sink()
                    .expect("Can't open default rodio stream");
                let player = Arc::new(rodio::Player::connect_new(stream_handle.mixer()));
                (Arc::new(stream_handle), player)
            },
            direction_of_slide_change: scroll_operations::Direction::Forward,
            scroll_id: Id::unique(),
            active_slide_id: Id::unique(),
            current_font: cosmic::font::default(),
            slide_action_map: None,
            obs_client: None,
            context_menu_id: None,
            context_point: Point::ORIGIN,
            obs_scenes: None,
            image_loader: ImageLoader::default(),
            preview_size: 100.0,
            animation: None,
            animator: None,
            now: Instant::now(),
            view_mode: ViewMode::Row,
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::NextSlide => {
                if let Some((item, slide)) = self.next_slide() {
                    self.direction_of_slide_change =
                        scroll_operations::Direction::Forward;
                    return self.update(Message::ActivateSlide(item, slide));
                }
            }
            Message::PrevSlide => {
                if let Some((item, slide)) = self.previous_slide() {
                    self.direction_of_slide_change =
                        scroll_operations::Direction::Backward;
                    return self.update(Message::ActivateSlide(item, slide));
                }
            }
            Message::ActivateSlide(item_index, slide_index) => {
                debug!(slide_index, item_index);
                if let Some(slide) = self
                    .service
                    .get(item_index)
                    .and_then(|item| item.slides.get(slide_index))
                {
                    self.current_item_index = item_index;
                    self.current_slide_index = slide_index;
                    return self.change_slide(slide.clone());
                }
            }
            Message::Tick(instant) => {
                self.now = instant;
            }
            Message::AddObsClient(client) => {
                self.obs_client = Some(client);
            }
            Message::RightClickSlide(item_index, slide_index) => {
                debug!(item_index, slide_index, "right clicked slide");
                self.context_menu_id = Some((item_index, slide_index));
                self.context_point = self.hovered_point.unwrap_or_default();
                if let Some(client) = &self.obs_client {
                    let client = Arc::clone(client);
                    return Action::Task(Task::perform(
                        async move { client.scenes().list().await },
                        |res| match res {
                            Ok(scenes) => Message::UpdateObsScenes(scenes.scenes),
                            Err(_) => todo!(),
                        },
                    ));
                }
            }
            Message::UpdateObsScenes(scenes) => {
                debug!(?scenes, "updating obs scenes");
                self.obs_scenes = Some(scenes);
            }
            Message::AssignObsScene(scene_index) => {
                let slide_id = self.context_menu_id.expect(
                    "In this match we should always already have a context menu id",
                );
                let Some(scenes) = &self.obs_scenes else {
                    return Action::None;
                };
                let new_scene = &scenes[scene_index];
                debug!(?scenes, ?new_scene, "updating obs actions");
                if let Some(map) = self.slide_action_map.as_mut() {
                    if let Some(actions) = map.get_mut(&slide_id) {
                        let mut altered_actions = vec![];
                        for action in actions.iter_mut() {
                            match action {
                                slide_actions::Action::Obs {
                                    action: ObsAction::Scene { .. },
                                } => altered_actions.push(slide_actions::Action::Obs {
                                    action: ObsAction::Scene {
                                        scene: new_scene.clone(),
                                    },
                                }),
                                _ => altered_actions.push(action.to_owned()),
                            }
                        }
                        *actions = altered_actions;
                        debug!("updating the obs scene {:?}", new_scene);
                    } else if map
                        .insert(
                            slide_id,
                            vec![slide_actions::Action::Obs {
                                action: ObsAction::Scene {
                                    scene: new_scene.clone(),
                                },
                            }],
                        )
                        .is_none()
                    {
                        debug!("adding the obs scene {:?}", new_scene);
                    } else {
                        debug!("updating the obs scene {:?}", new_scene);
                    }
                } else {
                    let mut map = HashMap::new();
                    map.insert(
                        slide_id,
                        vec![slide_actions::Action::Obs {
                            action: ObsAction::Scene {
                                scene: new_scene.clone(),
                            },
                        }],
                    );
                    self.slide_action_map = Some(map);
                }
            }
            Message::AssignSlideAction(action) => {
                let slide_id = self.context_menu_id.expect(
                    "In this match we should always already have a context menu id",
                );
                if let Some(map) = self.slide_action_map.as_mut() {
                    if let Some(actions) = map.get_mut(&slide_id) {
                        actions.push(action);
                    } else {
                        map.insert(slide_id, vec![action]);
                    }
                } else {
                    let mut map = HashMap::new();
                    map.insert(slide_id, vec![action]);
                    self.slide_action_map = Some(map);
                }
            }
            Message::ChangePreviewSize(size) => {
                self.preview_size = size as f32;
            }
            Message::ChangeFont(s) => {
                let font_name = s.into_boxed_str();
                let family = Family::Name(Box::leak(font_name));
                let weight = Weight::Normal;
                let stretch = Stretch::Normal;
                let style = Style::Normal;
                let font = Font {
                    family,
                    weight,
                    stretch,
                    style,
                };
                self.current_font = font;
            }
            Message::EndVideo => {
                // if self.current_slide.video_loop() {
                //     if let Some(video) = &mut self.video {
                //         match video.restart_stream() {
                //             Ok(_) => info!("Restarting video"),
                //             Err(e) => {
                //                 error!("Couldn't restart video: {e}")
                //             }
                //         }
                //     }
                // }
            }
            Message::StartVideo => {
                if let Some(video) = &mut self.preview_video {
                    video.set_paused(false);
                    video.set_looping(self.current_slide.video_loop());
                    debug!(?video);
                }
                if let Some(video) = &mut self.presentation_video {
                    video.set_paused(false);
                    video.set_looping(self.current_slide.video_loop());
                }
            }
            Message::PlayPauseVideo => {
                if let Some(video) = &mut self.preview_video {
                    video.set_paused(!video.paused());
                    video.set_looping(self.current_slide.video_loop());
                }
                if let Some(video) = &mut self.presentation_video {
                    video.set_paused(!video.paused());
                    video.set_looping(self.current_slide.video_loop());
                }
            }
            Message::LoadedService => {
                if let Some(item) = self.service.first()
                    && let Some(slide) = item.slides.first()
                {
                    let _ = self.change_slide(slide.clone());
                    self.sink.1.pause();
                    if let Some(video) = &mut self.preview_video {
                        video.set_paused(true);
                        video.set_looping(self.current_slide.video_loop());
                    }
                }
            }
            Message::VideoPos(position) => {
                if let Some(video) = &mut self.preview_video {
                    let position =
                        Position::Time(std::time::Duration::from_secs_f64(position));
                    match video.seek(position, false) {
                        Ok(()) => debug!("Video position changed: {:?}", position),
                        Err(e) => error!("Problem changing video position: {e}"),
                    }
                }
                if let Some(video) = &mut self.presentation_video {
                    let position =
                        Position::Time(std::time::Duration::from_secs_f64(position));
                    match video.seek(position, false) {
                        Ok(()) => debug!("Video position changed: {:?}", position),
                        Err(e) => error!("Problem changing video position: {e}"),
                    }
                }
            }
            Message::VideoFrame => {
                if let Some(video) = &self.preview_video
                    && self.video_position > 0.0
                    && video.position().as_secs_f64() != 0.0
                {
                    self.video_position = video.position().as_secs_f64();
                }
            }
            Message::MissingPlugin(element) => {
                if let Some(video) = &mut self.preview_video {
                    video.set_paused(true);
                }
                return Action::Task(Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            match gst_pbutils::MissingPluginMessage::parse(&element) {
                                Ok(missing_plugin) => {
                                    let mut install_ctx = gst_pbutils::InstallPluginsContext::new();
                                    install_ctx
                                        .set_desktop_id(&format!("{}.desktop", "org.chriscochrun.lumina"));
                                    let install_detail = missing_plugin.installer_detail();
                                    println!("installing plugins: {install_detail}");
                                    let status = gst_pbutils::missing_plugins::install_plugins_sync(
                                        &[&install_detail],
                                        Some(&install_ctx),
                                    );
                                    info!("plugin install status: {}", status);
                                    info!(
                                        "gstreamer registry update: {:?}",
                                        iced_video_player::gst::Registry::update()
                                    );
                                }
                                Err(err) => {
                                    warn!("failed to parse missing plugin message: {err}");
                                }
                            }
                            Message::None
                        })
                        .await
                        .expect("Spawning a task shouldn't fail")
                    },
                    |x| x,
                ));
            }
            Message::HoveredSlide(Some((item, slide, point))) => {
                self.hovered_slide = Some((item, slide));
                self.hovered_point = Some(point);
            }
            Message::HoveredSlide(None) => {
                self.hovered_slide = None;
                self.hovered_point = None;
            }
            Message::None => debug!("Presenter Message::None"),
            Message::Error(error) => {
                error!(error);
            }
            Message::CloseContextMenu => self.context_menu_id = None,
        }
        Action::None
    }

    pub fn view(&self) -> Element<Message> {
        let settings = crate::ui::widgets::slide::SlideSettings {
            delegate: false,
            hide_mouse: false,
            animation: self.animation.as_ref(),
            now: self.now,
        };

        let video: Option<Element<Message>> =
            self.presentation_video.as_ref().map(|video| {
                VideoPlayer::new(video)
                    .mouse_hidden(settings.hide_mouse)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .on_end_of_stream(Message::EndVideo)
                    .on_new_frame(Message::VideoFrame)
                    .on_missing_plugin(Message::MissingPlugin)
                    .on_warning(|w| Message::Error(w.to_string()))
                    .on_error(|e| Message::Error(e.to_string()))
                    .content_fit(ContentFit::Contain)
                    .into()
            });

        slide(
            &self.current_slide,
            self.old_slide.as_ref(),
            video,
            settings,
        )
        .animate({
            if let Some(animator) = &self.animator {
                let progress = animator.interpolate(0.0, 1.0, self.now);
                let prev_progress = animator.interpolate(1.0, 0.0, self.now);
                if progress >= 1.0 {
                    AnimationState::Idle
                } else {
                    AnimationState::Running {
                        direction: crate::ui::widgets::slide::Direction::Forward,
                        new_slide_progress: progress,
                        prev_slide_progress: prev_progress,
                    }
                }
            } else {
                AnimationState::Idle
            }
        })
        .into()
    }

    pub fn view_preview(&self) -> Element<Message> {
        let settings = crate::ui::widgets::slide::SlideSettings {
            delegate: false,
            hide_mouse: false,
            animation: self.animation.as_ref(),
            now: self.now,
        };
        let video: Option<Element<Message>> = self.preview_video.as_ref().map(|video| {
            VideoPlayer::new(video)
                .mouse_hidden(settings.hide_mouse)
                .width(Length::Fill)
                .height(Length::Fill)
                .on_end_of_stream(Message::EndVideo)
                .on_new_frame(Message::VideoFrame)
                .on_missing_plugin(Message::MissingPlugin)
                .on_warning(|w| Message::Error(w.to_string()))
                .on_error(|e| Message::Error(e.to_string()))
                .content_fit(ContentFit::Contain)
                .into()
        });

        slide(
            &self.current_slide,
            self.old_slide.as_ref(),
            video,
            settings,
        )
        .animate({
            if let Some(animator) = &self.animator {
                let progress = animator.interpolate(0.0, 1.0, self.now);
                let prev_progress = animator.interpolate(1.0, 0.0, self.now);
                if progress >= 1.0 {
                    AnimationState::Idle
                } else {
                    AnimationState::Running {
                        direction: crate::ui::widgets::slide::Direction::Forward,
                        new_slide_progress: progress,
                        prev_slide_progress: prev_progress,
                    }
                }
            } else {
                AnimationState::Idle
            }
        })
        .into()

        // slide_view(
        //     &self.current_slide,
        //     self.preview_video.as_ref(),
        //     SlideSettings {
        //         delegate: false,
        //         hide_mouse: false,
        //         previous_slide: self.old_slide.as_ref(),
        //         animation: self.animation.as_ref(),
        //         animator: self.animator.as_ref(),
        //         now: self.now,
        //     },
        // )
    }

    #[allow(clippy::too_many_lines)]
    pub fn preview_grid(&self) -> Element<Message> {
        let Spacing {
            space_none,
            space_xs,
            space_s,
            space_l,
            ..
        } = theme::spacing();
        // let mut grid = grid(vec![]).spacing(space_m);
        let mut items = vec![];
        for (item_index, item) in self.service.iter().enumerate() {
            let slides_length = item.slides.len();
            for (slide_index, slide) in item.slides.iter().enumerate() {
                let is_current_slide = (item_index, slide_index)
                    == (self.current_item_index, self.current_slide_index);

                let settings = crate::ui::widgets::slide::SlideSettings {
                    delegate: true,
                    hide_mouse: false,
                    animation: self.animation.as_ref(),
                    now: self.now,
                };

                let slide = widgets::slide::slide(
                    slide,
                    None,
                    None::<Element<Message>>,
                    settings,
                );

                let delegate = mouse_area(
                    Container::new(slide)
                        .id(if is_current_slide {
                            self.active_slide_id.clone()
                        } else {
                            Id::unique()
                        })
                        .style(move |t| {
                            let mut style = container::Style::default();
                            let theme = t.cosmic();
                            let hovered =
                                self.hovered_slide == Some((item_index, slide_index));
                            style.background =
                                Some(Background::Color(if is_current_slide {
                                    theme.accent.base.into()
                                } else if hovered {
                                    theme.accent.hover.into()
                                } else {
                                    theme.palette.neutral_3.into()
                                }));
                            style.border = Border::default().rounded(10.0);
                            style.shadow = Shadow {
                                color: Color::BLACK,
                                offset: {
                                    if is_current_slide || hovered {
                                        Vector::new(5.0, 5.0)
                                    } else {
                                        Vector::new(0.0, 0.0)
                                    }
                                },
                                blur_radius: {
                                    if is_current_slide || hovered {
                                        10.0
                                    } else {
                                        0.0
                                    }
                                },
                            };
                            style
                        })
                        .center_x(self.preview_size * 16.0 / 9.0)
                        .height(self.preview_size)
                        .padding(10),
                )
                .interaction(cosmic::iced::mouse::Interaction::Pointer)
                .on_move(move |point| {
                    Message::HoveredSlide(Some((item_index, slide_index, point)))
                })
                .on_exit(Message::HoveredSlide(None))
                .on_release(Message::ActivateSlide(item_index, slide_index))
                .on_right_release(Message::RightClickSlide(item_index, slide_index));
                let item = if slide_index == 0 {
                    let label = text::body(&item.title)
                        .ellipsize(Ellipsize::End(EllipsizeHeightLimit::Lines(1)))
                        .width(self.preview_size * 16.0 / 9.0);

                    column![label, delegate]
                        .align_x(Horizontal::Center)
                        .spacing(space_s)
                        .apply(container)
                } else {
                    delegate
                        .apply(container)
                        .padding([space_l, space_none, space_none, space_none])
                };
                items.push(self.context_menu((item_index, slide_index), item.into()));

                items.push(
                    container(space::vertical().width(space_s))
                        .class(if slide_index + 1 == slides_length {
                            theme::Container::Card
                        } else {
                            theme::Container::WindowBackground
                        })
                        .padding([space_none, space_xs, space_none, space_xs])
                        .into(),
                );
            }
        }
        let scrollable = scrollable(
            container(
                flex_row(items)
                    .justify_content(Some(JustifyContent::FlexStart))
                    .align_items(cosmic::iced::Alignment::Center)
                    .justify_items(cosmic::iced::Alignment::End)
                    .column_spacing(space_s)
                    .row_spacing(space_s),
            )
            .style(|_t| {
                let style = container::Style::default();
                style.border(Border::default().width(2))
            }),
        )
        .auto_scroll(true)
        // .direction(Direction::Horizontal(Scrollbar::new()))
        .id(self.scroll_id.clone())
        .height(Length::Fill)
        .width(Length::Fill);
        scrollable.into()

        // scrollable.into()
    }

    #[allow(clippy::too_many_lines)]
    pub fn preview_bar(&self) -> Element<Message> {
        let Spacing {
            space_xs, space_s, ..
        } = theme::spacing();
        let mut items = vec![];
        self.service
            .iter()
            .enumerate()
            .for_each(|(item_index, item)| {
                let mut slides = vec![];
                item.slides
                    .iter()
                    .enumerate()
                    .for_each(|(slide_index, slide)| {
                        let is_current_slide = (item_index, slide_index)
                            == (self.current_item_index, self.current_slide_index);

                        let settings = crate::ui::widgets::slide::SlideSettings {
                            delegate: true,
                            hide_mouse: false,
                            animation: self.animation.as_ref(),
                            now: self.now,
                        };

                        let container = widgets::slide::slide(
                            slide,
                            None,
                            None::<Element<Message>>,
                            settings,
                        );

                        // let container = slide_view(
                        //     slide,
                        //     self.preview_video.as_ref(),
                        //     SlideSettings {
                        //         delegate: true,
                        //         hide_mouse: false,
                        //         previous_slide: None,
                        //         animation: None,
                        //         animator: None,
                        //         now: self.now,
                        //     },
                        // );
                        let delegate = mouse_area(
                            Container::new(container)
                                .id(if is_current_slide {
                                    self.active_slide_id.clone()
                                } else {
                                    Id::unique()
                                })
                                .style(move |t| {
                                    let mut style = container::Style::default();
                                    let theme = t.cosmic();
                                    let hovered = self.hovered_slide
                                        == Some((item_index, slide_index));
                                    style.background =
                                        Some(Background::Color(if is_current_slide {
                                            theme.accent.base.into()
                                        } else if hovered {
                                            theme.accent.hover.into()
                                        } else {
                                            theme.palette.neutral_3.into()
                                        }));
                                    style.border = Border::default().rounded(10.0);
                                    style.shadow = Shadow {
                                        color: Color::BLACK,
                                        offset: {
                                            if is_current_slide || hovered {
                                                Vector::new(5.0, 5.0)
                                            } else {
                                                Vector::new(0.0, 0.0)
                                            }
                                        },
                                        blur_radius: {
                                            if is_current_slide || hovered {
                                                10.0
                                            } else {
                                                0.0
                                            }
                                        },
                                    };
                                    style
                                })
                                .center_x(self.preview_size * 16.0 / 9.0)
                                .height(self.preview_size)
                                .padding(10),
                        )
                        .interaction(cosmic::iced::mouse::Interaction::Pointer)
                        .on_move(move |point| {
                            Message::HoveredSlide(Some((item_index, slide_index, point)))
                        })
                        .on_exit(Message::HoveredSlide(None))
                        .on_release(Message::ActivateSlide(item_index, slide_index))
                        .on_right_release(
                            Message::RightClickSlide(item_index, slide_index),
                        );
                        let context_menu =
                            self.context_menu((item_index, slide_index), delegate.into());
                        slides.push(context_menu);
                    });
                let slides_length = slides.len() as f32;
                let row = Row::from_vec(slides)
                    .spacing(space_s)
                    .padding([0, 15, 0, 15]);
                let label = text::body(item.title.clone())
                    .ellipsize(Ellipsize::End(EllipsizeHeightLimit::Lines(1)))
                    .width(self.preview_size * slides_length * 16.0 / 9.0);
                let label_container = container(label).padding([0, 0, 0, 15]);
                let divider = vertical::light().width(2);
                items.push(
                    container(column![label_container, row].spacing(space_s))
                        .padding(space_xs)
                        .into(),
                );
                items.push(divider.into());
            });
        let scrollable = scrollable(container(Row::from_vec(items)).style(|_t| {
            let style = container::Style::default();
            style.border(Border::default().width(2))
        }))
        .direction(Direction::Horizontal(Scrollbar::new()))
        .height(Length::Fill)
        // .width(Length::Fill)
        .id(self.scroll_id.clone());
        scrollable.into()
    }

    fn context_menu<'a>(
        &'a self,
        id: (usize, usize),
        items: Element<'a, Message>,
    ) -> Element<'a, Message> {
        if self
            .context_menu_id
            .is_some_and(|context_id| context_id == id)
        {
            let menu_item = |label, message| {
                menu::menu_button(vec![text(label).into(), space::horizontal().into()])
                    .on_press(message)
            };
            let obs_scenes = self.obs_scenes.as_ref().map(|scenes| {
                scenes.iter().map(|scene| {
                    menu_item(
                        scene.id.name.as_ref(),
                        Message::AssignObsScene(scene.index),
                    )
                })
            });

            let mut menu_items: Vec<Element<Message>> = vec![
                menu_item(
                    "Start Stream",
                    Message::AssignSlideAction(slide_actions::Action::Obs {
                        action: ObsAction::StartStream,
                    }),
                )
                .into(),
                menu_item(
                    "Stop Stream",
                    Message::AssignSlideAction(slide_actions::Action::Obs {
                        action: ObsAction::StopStream,
                    }),
                )
                .into(),
            ];
            if let Some(scenes) = obs_scenes {
                menu_items.push(divider::horizontal::light().into());
                menu_items.push(
                    text("Scenes")
                        .class(theme::Text::Color(
                            theme::active().cosmic().palette.neutral_7.into(),
                        ))
                        .align_x(Alignment::Center)
                        .apply(container)
                        .padding(theme::spacing().space_s)
                        .into(),
                );
                menu_items.push(divider::horizontal::light().into());
                for scene in scenes {
                    menu_items.push(scene.into());
                }
            }

            let item_column = column::with_children(menu_items)
                .spacing(theme::spacing().space_s)
                .apply(container)
                .width(300)
                .padding(theme::spacing().space_s)
                .class(theme::Container::Dropdown);

            popover(items)
                .position(popover::Position::Point(self.context_point))
                .on_close(Message::CloseContextMenu)
                .popup(item_column)
                .into()
        } else {
            items
        }
    }

    fn reset_video(&mut self) {
        if self.current_slide.background().kind == BackgroundKind::Video
            && let path = &self.current_slide.background().path
            && path.exists()
        {
            let url =
                Url::from_file_path(path).expect("There should be a video file here");

            let video_settings = VideoSettings {
                mute: true,
                framerate: 30,
                appsink_name: "lumina_preview".to_string(),
            };
            match gst_video::create_video(&url, &video_settings) {
                Ok(mut v) => {
                    v.set_looping(self.current_slide.video_loop());
                    v.set_muted(true);
                    self.preview_video = Some(v);
                }
                Err(e) => {
                    error!("Had an error creating the video object: {e}");
                    self.preview_video = None;
                }
            }
            let video_settings = VideoSettings {
                mute: false,
                framerate: 60,
                appsink_name: "lumina_video".to_string(),
            };
            match gst_video::create_video(&url, &video_settings) {
                Ok(mut v) => {
                    v.set_looping(self.current_slide.video_loop());
                    self.presentation_video = Some(v);
                }
                Err(e) => {
                    error!("Had an error creating the video object: {e}");
                    self.presentation_video = None;
                }
            }
        } else {
            self.preview_video = None;
            self.presentation_video = None;
        }
    }

    pub fn update_items(&mut self, items: Arc<Vec<ServiceItem>>) {
        let total_slides: usize = items.iter().fold(0, |a, item| a + item.slides.len());
        self.service = items;
        self.total_slides = total_slides;
    }

    pub fn run_slide_actions(&self) -> Task<Message> {
        let mut tasks = vec![];

        if let Some(map) = &self.slide_action_map
            && let Some(actions) =
                map.get(&(self.current_item_index, self.current_slide_index))
        {
            for action in actions {
                match action {
                    slide_actions::Action::Obs { action } => {
                        debug!("found obs slide actions");
                        if let Some(obs) = &self.obs_client {
                            let obs = Arc::clone(obs);
                            let action = action.to_owned();
                            let task = Task::perform(
                                async move { action.run(obs).await },
                                |res| {
                                    debug!(?res);
                                    Message::None
                                },
                            );
                            tasks.push(task);
                        }
                    }
                    slide_actions::Action::Other => todo!(),
                }
            }
        }
        Task::batch(tasks)
    }

    #[allow(unused)]
    fn load_images(&mut self) {
        if matches!(self.current_slide.background.kind, BackgroundKind::Image) {
            let path = &self.current_slide.background().path;
            self.current_slide.background.image_handle =
                match self.image_loader.get_image(path) {
                    Ok(image) => Some(image),
                    Err(e) => {
                        error!("Couldn't load image: {e:?}");
                        None
                    }
                }
        }

        if let Some(slide) = self.activating_slide.as_mut()
            && matches!(slide.background().kind, BackgroundKind::Image)
        {
            let path = &slide.background().path;
            let res_handle = self.image_loader.load_image(path);
            slide.background.image_handle = match res_handle {
                Ok(image) => Some(image),
                Err(e) => {
                    error!("Couldn't load image: {e:?}");
                    None
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn change_slide(&mut self, slide: Slide) -> Action {
        let slide_text = slide.text();
        debug!(slide_text, "slide changed");
        let bg = slide.background().clone();
        debug!(?bg, "comparing background...");
        let backgrounds_match = self.current_slide.background() == slide.background();

        debug!("cloning slide...");
        let font = slide
            .font()
            .map_or_else(String::new, |font| font.get_name());
        let _ = self.update(Message::ChangeFont(font));
        debug!("changing video now...");
        self.old_slide = Some(self.current_slide.clone());
        self.current_slide = slide;
        if !backgrounds_match {
            if let Some(video) = &mut self.preview_video {
                video.set_paused(true);
            }
            if let Some(video) = &mut self.presentation_video {
                video.set_paused(true);
            }
            self.reset_video();
        }


        if let Some(item) = self.service.get(self.current_item_index)
            && let Some(animation) = item.animation.clone()
        {
            self.animator = Some(animation.get_animator(self.now));
            self.animation = Some(animation);
        } else {
            self.animation = None;
            self.animator = None;
        }


        let mut tasks = vec![];
        #[allow(clippy::cast_precision_loss)]
        match self.view_mode {
            ViewMode::Grid => {
                tasks.push(focus_target(
                    self.scroll_id.clone(),
                    Some(self.active_slide_id.clone()),
                    self.direction_of_slide_change.clone(),
                ));
            }
            ViewMode::Row => {
                tasks.push(focus_target(
                    self.scroll_id.clone(),
                    Some(self.active_slide_id.clone()),
                    self.direction_of_slide_change.clone(),
                ));
            }
            ViewMode::Detail => todo!(),
        };

        if self.slide_action_map.is_some() {
            debug!("Found slide actions, running them");
            tasks.push(self.run_slide_actions());
        }

        if self.current_slide.audio() != self.audio {
            self.audio = self.current_slide.audio();
            self.sink.1.stop();
            self.sink.1.clear();
            self.audio_duration = None;
            if let Some(audio) = &self.audio
                && audio.exists()
            {
                let file = BufReader::new(
                    File::open(audio).expect("There should be an audio file here"),
                );
                let source =
                    Decoder::new(file).expect("There should be an audio decoder here");
                self.audio_duration = source.total_duration();
                self.sink.1.append(source);
                self.sink.1.play();
                self.audio_position = Some(self.sink.1.get_pos());
            }
        }
        let task_count = tasks.len();
        debug!(?task_count);
        Action::Task(Task::batch(tasks))
    }

    pub(crate) fn preview_size(&self) -> f64 {
        self.preview_size.into()
    }

    pub(crate) fn previous_slide(&self) -> Option<(usize, usize)> {
        if self.current_item_index == 0 && self.current_slide_index == 0 {
            return None;
        } else if self.current_slide_index == 0 {
            let target_item = self.current_item_index - 1;
            let last_slide = self
                .service
                .get(target_item)
                .map(|i| i.slides.len() - 1)
                .expect("We have checked that this item should be here.");
            return Some((target_item, last_slide));
        }
        let target_slide = self.current_slide_index - 1;
        Some((self.current_item_index, target_slide))
    }

    pub(crate) fn next_slide(&self) -> Option<(usize, usize)> {
        if self
            .service
            .get(self.current_item_index)
            .is_some_and(|i| i.slides.len() == self.current_slide_index + 1)
        {
            return Some((self.current_item_index + 1, 0));
        } else if self
            .service
            .get(self.current_item_index)
            .is_some_and(|i| i.slides.len() > self.current_slide_index + 1)
        {
            return Some((self.current_item_index, self.current_slide_index + 1));
        }
        None
    }
}

pub struct SlideSettings<'a> {
    pub delegate: bool,
    pub hide_mouse: bool,
    pub previous_slide: Option<&'a Slide>,
    pub animation: Option<&'a crate::core::animation::Animation>,
    pub animator: Option<&'a Animation<bool>>,
    pub now: Instant,
}

#[allow(clippy::too_many_lines)]
pub(crate) fn slide_view<'a>(
    slide: &'a Slide,
    video: Option<&'a Video>,
    settings: SlideSettings<'a>,
) -> Element<'a, Message> {
    responsive(move |size| {
        let width = size.height * 16.0 / 9.0;
        let black = Container::new(Space::new())
            .style(|_| container::background(Background::Color(Color::BLACK)))
            .clip(true)
            .width(width)
            .height(Length::Fill);
        let mut stack = stack(vec![black.into()]);

        match slide.background().kind {
            BackgroundKind::Image => {
                stack =
                    stack.push(slide.background().image_allocation.as_ref().map_or_else(
                        || {
                            cosmic_image(&slide.background().path)
                                .content_fit(ContentFit::Contain)
                                .width(width)
                                .height(Length::Fill)
                                .apply(container)
                                .center(Length::Fill)
                                .clip(true)
                        },
                        |allocation| {
                            loaded_image(
                                allocation.handle(),
                                cosmic_image(allocation.handle())
                                    .content_fit(ContentFit::Contain)
                                    .width(width)
                                    .height(Length::Fill),
                            )
                            .apply(container)
                            .center(Length::Fill)
                            .clip(true)
                        },
                    ));
            }
            BackgroundKind::Video => {
                if let Some(video) = &video
                    && !settings.delegate
                {
                    stack = stack.push(
                        Container::new(
                            VideoPlayer::new(video)
                                .mouse_hidden(settings.hide_mouse)
                                .width(width)
                                .height(Length::Fill)
                                .on_end_of_stream(Message::EndVideo)
                                .on_new_frame(Message::VideoFrame)
                                .on_missing_plugin(Message::MissingPlugin)
                                .on_warning(|w| Message::Error(w.to_string()))
                                .on_error(|e| Message::Error(e.to_string()))
                                .content_fit(ContentFit::Contain),
                        )
                        .center(Length::Fill)
                        .clip(true),
                    );
                } else if settings.delegate {
                    stack = stack.push(slide.thumbnail.as_ref().map_or_else(
                        || {
                            Container::new(space::horizontal())
                                .center(Length::Fill)
                                .clip(true)
                        },
                        |allocation| {
                            loaded_image(
                                allocation.handle(),
                                cosmic_image(allocation.handle())
                                    .content_fit(ContentFit::Contain)
                                    .width(width)
                                    .height(Length::Fill),
                            )
                            .apply(container)
                            .center(Length::Fill)
                            .clip(true)
                        },
                    ));
                }
            }
            BackgroundKind::Pdf => {
                if let Some(pdf) = slide.pdf_page() {
                    stack = stack.push(
                        Container::new(loaded_image(
                            &pdf,
                            cosmic_image(&pdf)
                                .width(width)
                                .height(Length::Fill)
                                .content_fit(ContentFit::Contain),
                        ))
                        .center(Length::Fill)
                        .clip(true),
                    );
                }
            }
            BackgroundKind::Html => todo!(),
        }
        if let Some(text) = &slide.text_svg
            && let Some(handle) = &text.handle
        {
            if let Some(slide) = settings.previous_slide
                && let Some(text) = &slide.text_svg
                && let Some(handle) = &text.handle
                && !settings.delegate
                && settings.animation.is_some()
            {
                let mut column = column(Vec::new());
                column = column.push(loaded_image(
                    handle,
                    cosmic_image(handle)
                        .content_fit(ContentFit::Contain)
                        .opacity({
                            if let Some(animator) = settings.animator
                                && let Some(animation) = settings.animation
                            {
                                if matches!(
                                    animation,
                                    crate::core::animation::Animation::CrossFade { .. }
                                ) {
                                    animator.interpolate(1.0, 0.0, settings.now)
                                } else {
                                    if animator.is_animating(settings.now) {
                                        1.0
                                    } else {
                                        0.0
                                    }
                                }
                            } else {
                                1.0
                            }
                        })
                        .width(Length::Fill)
                        .height(size.height),
                ));
                if let Some(animator) = settings.animator
                    && let Some(animation) = settings.animation
                    && matches!(
                        animation,
                        crate::core::animation::Animation::SlideUp { .. }
                    )
                {
                    column = column.push(space::vertical().height({
                        animator.interpolate(0.0, size.height, settings.now)
                    }));
                };
                stack = stack.push(
                    column
                        .apply(scrollable)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .scrollbar_width(0)
                        .scroller_width(0),
                );
            }
            let mut column = column(Vec::new());
            if let Some(animator) = settings.animator
                && let Some(animation) = settings.animation
                && matches!(animation, crate::core::animation::Animation::SlideUp { .. })
            {
                column = column
                    .push(space::vertical().height({
                        animator.interpolate(size.height, 0.0, settings.now)
                    }));
            }
            column = column.push(loaded_image(
                handle,
                cosmic_image(handle)
                    .content_fit(ContentFit::Cover)
                    .opacity({
                        if let Some(animator) = settings.animator
                            && let Some(animation) = settings.animation
                            && matches!(
                                animation,
                                crate::core::animation::Animation::CrossFade { .. }
                            )
                        {
                            animator.interpolate(0.0, 1.0, settings.now)
                        } else {
                            1.0
                        }
                    })
                    .width(Length::Fill)
                    .height(size.height),
            ));
            stack = stack.push(
                column
                    .apply(scrollable)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .scrollbar_width(0)
                    .scroller_width(0),
            );
        }
        Container::new(stack).center(Length::Fill).into()
    })
    .into()
}

#[cfg(test)]
mod test {
    use crate::core::presentations::{PresKind, Presentation};
    use crate::core::slide::TextAlignment;
    use crate::core::songs::{Song, VerseName};

    use super::*;
    use pretty_assertions::assert_eq;
    use sqlx::types::chrono::Local;

    #[test]
    fn test_next_slide() {
        let service = test_service();
        let mut presenter = Presenter::with_items(Arc::new(service));
        presenter.update(Message::NextSlide);
        dbg!(&presenter.service);
        assert_eq!(presenter.current_item_index, 1);
        assert_eq!(presenter.current_slide_index, 0);
        presenter.update(Message::NextSlide);
        assert_eq!(presenter.current_item_index, 1);
        assert_eq!(presenter.current_slide_index, 1);
        presenter.update(Message::NextSlide);
        assert_eq!(presenter.current_item_index, 1);
        assert_eq!(presenter.current_slide_index, 2);
        presenter.update(Message::PrevSlide);
        assert_eq!(presenter.current_item_index, 1);
        assert_eq!(presenter.current_slide_index, 1);
        presenter.update(Message::PrevSlide);
        assert_eq!(presenter.current_item_index, 1);
        assert_eq!(presenter.current_slide_index, 0);
        presenter.update(Message::PrevSlide);
        assert_eq!(presenter.current_item_index, 0);
        assert_eq!(presenter.current_slide_index, 0);
    }

    fn test_service() -> Vec<ServiceItem> {
        let mut service = Vec::new();
        let song = test_song();
        let video = test_video("Christ Nutshell".into());
        let presentation = test_presentation();
        let mut video_item = ServiceItem::from(&video);
        video_item.slides = video_item.to_slides().expect("");
        let mut song_item = ServiceItem::from(&song);
        song_item.slides = song_item.to_slides().expect("");
        let mut pres_item = ServiceItem::from(&presentation);
        pres_item.slides = pres_item.to_slides().expect("");
        service.push(video_item);
        service.push(song_item);
        service.push(pres_item);
        service
    }

    fn test_video(title: String) -> crate::core::videos::Video {
        crate::core::videos::Video {
            title,
            path: PathBuf::from(
                "/home/chris/nc/tfc/Documents/lessons/videos/christ-nutshell.mp4",
            ),
            ..Default::default()
        }
    }

    pub fn test_song() -> Song {
        let lyrics = "Some({Verse(number:4):\"Our Savior displayed\\nOn a criminal\\'s cross\\n\\nDarkness rejoiced as though\\nHeaven had lost\\n\\nBut then Jesus arose\\nWith our freedom in hand\\n\\nThat\\'s when death was arrested\\nAnd my life began\\n\\nThat\\'s when death was arrested\\nAnd my life began\",Intro(number:1):\"Death Was Arrested\\nNorth Point Worship\",Verse(number:3):\"Released from my chains,\\nI\\'m a prisoner no more\\n\\nMy shame was a ransom\\nHe faithfully bore\\n\\nHe cancelled my debt and\\nHe called me His friend\\n\\nWhen death was arrested\\nAnd my life began\",Bridge(number:1):\"Oh, we\\'re free, free,\\nForever we\\'re free\\n\\nCome join the song\\nOf all the redeemed\\n\\nYes, we\\'re free, free,\\nForever amen\\n\\nWhen death was arrested\\nAnd my life began\\n\\nOh, we\\'re free, free,\\nForever we\\'re free\\n\\nCome join the song\\nOf all the redeemed\\n\\nYes, we\\'re free, free,\\nForever amen\\n\\nWhen death was arrested\\nAnd my life began\",Other(number:99):\"When death was arrested\\nAnd my life began\\n\\nThat\\'s when death was arrested\\nAnd my life began\",Verse(number:2):\"Ash was redeemed\\nOnly beauty remains\\n\\nMy orphan heart\\nWas given a name\\n\\nMy mourning grew quiet,\\nMy feet rose to dance\\n\\nWhen death was arrested\\nAnd my life began\",Verse(number:1):\"Alone in my sorrow\\nAnd dead in my sin\\n\\nLost without hope\\nWith no place to begin\\n\\nYour love made a way\\nTo let mercy come in\\n\\nWhen death was arrested\\nAnd my life began\",Chorus(number:1):\"Oh, Your grace so free,\\nWashes over me\\n\\nYou have made me new,\\nNow life begins with You\\n\\nIt\\'s Your endless love,\\nPouring down on us\\n\\nYou have made us new,\\nNow life begins with You\"})".to_string();
        let verse_map: Option<HashMap<VerseName, String>> =
            ron::from_str(&lyrics).expect("");
        Song {
            id: 7,
            title: "Death Was Arrested".to_string(),
            lyrics: Some(lyrics),
            author: Some(
                "North Point Worship".to_string(),
            ),
            ccli: None,
            audio: Some("file:///home/chris/music/North Point InsideOut/Nothing Ordinary, Pt. 1 (Live)/05 Death Was Arrested (feat. Seth Condrey).mp3".into()),
            verse_order: Some(vec!["Some([Chorus(number:1),Intro(number:1),Other(number:99),Bridge(number:1),Verse(number:4),Verse(number:2),Verse(number:3),Verse(number:1)])".to_string()]),
            background: Some(crate::core::slide::Background::try_from("file:///home/chris/nc/tfc/openlp/Flood/motions/Ocean_Floor_HD.mp4").expect("")),
            text_alignment: Some(TextAlignment::MiddleCenter),
            font: Some("Quicksand Bold".to_string()),
            font_size: Some(80),
            stroke_size: None,
            verses: Some(vec![VerseName::Chorus { number: 1 }, VerseName::Intro { number: 1 }, VerseName::Other { number: 99 }, VerseName::Bridge { number: 1 }, VerseName::Verse { number: 4 }, VerseName::Verse { number: 2 }, VerseName::Verse { number: 3 }, VerseName::Verse { number: 1 }
            ]),
            verse_map,
            ..Default::default()
        }
    }

    fn test_presentation() -> Presentation {
        Presentation {
            id: 4,
            title: "mzt52.pdf".into(),
            path: PathBuf::from("/home/chris/docs/mzt52.pdf"),
            kind: PresKind::Pdf {
                starting_index: 0,
                ending_index: 67,
            },
            created_at: Local::now(),
            accessed_at: Local::now(),
        }
    }
}
