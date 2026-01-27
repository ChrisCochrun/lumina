use miette::{IntoDiagnostic, Result};
use obws::{Client, responses::scenes::Scene};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use cosmic::{
    Task,
    iced::{
        Background, Border, Color, ContentFit, Font, Length, Shadow,
        Vector,
        font::{Family, Stretch, Style, Weight},
    },
    iced_widget::{
        scrollable::{
            AbsoluteOffset, Direction, Scrollbar, scroll_to,
        },
        stack, vertical_rule,
    },
    prelude::*,
    widget::{
        Container, Id, Row, Space, container, context_menu, image,
        menu, mouse_area, responsive, scrollable, text,
    },
};
use iced_video_player::{Position, Video, VideoPlayer, gst_pbutils};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use tracing::{debug, error, info, warn};
use url::Url;

use crate::{
    BackgroundKind,
    core::{
        service_items::ServiceItem,
        slide::Slide,
        slide_actions::{self, ObsAction},
    },
};

const REFERENCE_WIDTH: f32 = 1920.0;
static DEFAULT_SLIDE: LazyLock<Slide> = LazyLock::new(Slide::default);

// #[derive(Default, Clone, Debug)]
pub(crate) struct Presenter {
    pub service: Vec<ServiceItem>,
    pub current_slide: Slide,
    pub current_item: usize,
    pub current_slide_index: usize,
    pub absolute_slide_index: usize,
    pub total_slides: usize,
    pub video: Option<Video>,
    pub video_position: f32,
    pub audio: Option<PathBuf>,
    sink: (Arc<Sink>, OutputStream),
    hovered_slide: Option<(usize, usize)>,
    scroll_id: Id,
    current_font: Font,
    slide_action_map:
        Option<HashMap<(usize, usize), Vec<slide_actions::Action>>>,
    obs_client: Option<Arc<Client>>,
    context_menu_id: Option<(usize, usize)>,
    obs_scenes: Option<Vec<Scene>>,
}

pub(crate) enum Action {
    Task(Task<Message>),
    NextSlide,
    PrevSlide,
    ChangeSlide(usize, usize),
    None,
}

#[derive(Clone)]
pub(crate) enum Message {
    NextSlide,
    PrevSlide,
    SlideChange(Slide),
    ActivateSlide(usize, usize),
    ClickSlide(usize, usize),
    EndVideo,
    StartVideo,
    StartAudio,
    EndAudio,
    VideoPos(f32),
    VideoFrame,
    MissingPlugin(gstreamer::Message),
    HoveredSlide(Option<(usize, usize)>),
    ChangeFont(String),
    Error(String),
    None,
    RightClickSlide(usize, usize),
    AssignObsScene(usize),
    UpdateObsScenes(Vec<Scene>),
    AddObsClient(Arc<Client>),
    AssignSlideAction(slide_actions::Action),
}

impl std::fmt::Debug for Message {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::NextSlide => write!(f, "NextSlide"),
            Self::PrevSlide => write!(f, "PrevSlide"),
            Self::SlideChange(arg0) => {
                f.debug_tuple("SlideChange").field(arg0).finish()
            }
            Self::ActivateSlide(arg0, arg1) => f
                .debug_tuple("ActivateSlide")
                .field(arg0)
                .field(arg1)
                .finish(),
            Self::ClickSlide(arg0, arg1) => f
                .debug_tuple("ClickSlide")
                .field(arg0)
                .field(arg1)
                .finish(),
            Self::EndVideo => write!(f, "EndVideo"),
            Self::StartVideo => write!(f, "StartVideo"),
            Self::StartAudio => write!(f, "StartAudio"),
            Self::EndAudio => write!(f, "EndAudio"),
            Self::VideoPos(arg0) => {
                f.debug_tuple("VideoPos").field(arg0).finish()
            }
            Self::VideoFrame => write!(f, "VideoFrame"),
            Self::MissingPlugin(arg0) => {
                f.debug_tuple("MissingPlugin").field(arg0).finish()
            }
            Self::HoveredSlide(arg0) => {
                f.debug_tuple("HoveredSlide").field(arg0).finish()
            }
            Self::ChangeFont(arg0) => {
                f.debug_tuple("ChangeFont").field(arg0).finish()
            }
            Self::Error(arg0) => {
                f.debug_tuple("Error").field(arg0).finish()
            }
            Self::None => write!(f, "None"),
            Self::RightClickSlide(arg0, arg1) => f
                .debug_tuple("RightClickSlide")
                .field(arg0)
                .field(arg1)
                .finish(),
            Self::AssignObsScene(arg0) => {
                f.debug_tuple("ObsSceneAssign").field(arg0).finish()
            }
            Self::UpdateObsScenes(arg0) => {
                f.debug_tuple("UpdateObsScenes").field(arg0).finish()
            }
            Self::AddObsClient(_) => write!(f, "AddObsClient"),
            Self::AssignSlideAction(action) => f
                .debug_tuple("AssignSlideAction")
                .field(action)
                .finish(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuAction {
    ObsSceneAssign(usize),
    ObsStartStream,
    ObsStopStream,
    ObsStartRecord,
    ObsStopRecord,
}

impl menu::Action for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::ObsSceneAssign(scene) => {
                Message::AssignObsScene(*scene)
            }
            MenuAction::ObsStartStream => Message::AssignSlideAction(
                slide_actions::Action::Obs {
                    action: ObsAction::StartStream,
                },
            ),
            MenuAction::ObsStopStream => Message::AssignSlideAction(
                slide_actions::Action::Obs {
                    action: ObsAction::StopStream,
                },
            ),
            MenuAction::ObsStartRecord => todo!(),
            MenuAction::ObsStopRecord => todo!(),
        }
    }
}

impl Presenter {
    fn create_video(url: Url) -> Result<Video> {
        // Based on `iced_video_player::Video::new`,
        // but without a text sink so that the built-in subtitle functionality triggers.
        use gstreamer as gst;
        use gstreamer_app as gst_app;
        use gstreamer_app::prelude::*;

        gst::init().into_diagnostic()?;

        let pipeline = format!(
            r#"playbin uri="{}" video-sink="videoscale ! videoconvert ! appsink name=lumina_video drop=true caps=video/x-raw,format=NV12,pixel-aspect-ratio=1/1""#,
            url.as_str()
        );

        let pipeline = gst::parse::launch(pipeline.as_ref())
            .into_diagnostic()?;
        let pipeline = pipeline
            .downcast::<gst::Pipeline>()
            .map_err(|_| iced_video_player::Error::Cast)
            .into_diagnostic()?;

        let video_sink: gst::Element =
            pipeline.property("video-sink");
        let pad = video_sink.pads().first().cloned().unwrap();
        let pad = pad.dynamic_cast::<gst::GhostPad>().unwrap();
        let bin = pad
            .parent_element()
            .unwrap()
            .downcast::<gst::Bin>()
            .unwrap();
        let video_sink = bin.by_name("lumina_video").unwrap();
        let video_sink =
            video_sink.downcast::<gst_app::AppSink>().unwrap();
        let result =
            Video::from_gst_pipeline(pipeline, video_sink, None);
        result.into_diagnostic()
    }

    pub fn with_items(items: Vec<ServiceItem>) -> Self {
        let video = {
            if let Some(item) = items.first() {
                if let Some(slide) = item.slides.first() {
                    let path = slide.background().path.clone();
                    if path.exists() {
                        let url = Url::from_file_path(path).unwrap();
                        let result = Video::new(&url);
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
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };
        let total_slides: usize =
            items.iter().fold(0, |a, item| a + item.slides.len());

        let slide =
            items.first().and_then(|item| item.slides.first());
        let audio = items
            .first()
            .and_then(|item| {
                item.slides.first().map(|slide| slide.audio())
            })
            .flatten();

        Self {
            current_slide: slide.unwrap_or(&DEFAULT_SLIDE).clone(),
            current_item: 0,
            current_slide_index: 0,
            absolute_slide_index: 0,
            total_slides,
            video,
            audio,
            service: items,
            video_position: 0.0,
            hovered_slide: None,
            sink: {
                let stream_handle =
                    OutputStreamBuilder::open_default_stream()
                        .expect("Can't open default rodio stream");
                (
                    Arc::new(Sink::connect_new(
                        stream_handle.mixer(),
                    )),
                    stream_handle,
                )
            },
            scroll_id: Id::unique(),
            current_font: cosmic::font::default(),
            slide_action_map: None,
            obs_client: None,
            context_menu_id: None,
            obs_scenes: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::AddObsClient(client) => {
                self.obs_client = Some(client);
            }
            Message::NextSlide => {
                return Action::NextSlide;
            }
            Message::PrevSlide => {
                return Action::PrevSlide;
            }
            Message::ClickSlide(item_index, slide_index) => {
                return Action::ChangeSlide(item_index, slide_index);
            }
            Message::RightClickSlide(item_index, slide_index) => {
                debug!(
                    item_index,
                    slide_index, "right clicked slide"
                );
                self.context_menu_id =
                    Some((item_index, slide_index));
                if let Some(client) = &self.obs_client {
                    let client = Arc::clone(client);
                    return Action::Task(Task::perform(
                        async move { client.scenes().list().await },
                        |res| match res {
                            Ok(scenes) => Message::UpdateObsScenes(
                                scenes.scenes,
                            ),
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
                let Some(scenes) = &self.obs_scenes else {
                    return Action::None;
                };
                let new_scene = &scenes[scene_index];
                debug!(?scenes, ?new_scene, "updating obs actions");
                if let Some(map) = self.slide_action_map.as_mut() {
                    if let Some(actions) = map.get_mut(
                        &self.context_menu_id.unwrap_or_default(),
                    ) {
                        let mut altered_actions = vec![];
                        actions.iter_mut().for_each(|action| {
                            match action {
                                slide_actions::Action::Obs {
                                    action: ObsAction::Scene { .. },
                                } => altered_actions.push(
                                    slide_actions::Action::Obs {
                                        action: ObsAction::Scene {
                                            scene: new_scene.clone(),
                                        },
                                    },
                                ),
                                _ => altered_actions
                                    .push(action.to_owned()),
                            }
                        });
                        *actions = altered_actions;
                        debug!(
                            "updating the obs scene {:?}",
                            new_scene
                        )
                    } else {
                        if map
                            .insert(
                                self.context_menu_id.unwrap(),
                                vec![slide_actions::Action::Obs {
                                    action: ObsAction::Scene {
                                        scene: new_scene.clone(),
                                    },
                                }],
                            )
                            .is_none()
                        {
                            debug!(
                                "adding the obs scene {:?}",
                                new_scene
                            )
                        } else {
                            debug!(
                                "updating the obs scene {:?}",
                                new_scene
                            )
                        }
                    }
                } else {
                    let mut map = HashMap::new();
                    map.insert(
                        self.context_menu_id.unwrap().clone(),
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
                if let Some(map) = self.slide_action_map.as_mut() {
                    if let Some(actions) =
                        map.get_mut(&self.context_menu_id.unwrap())
                    {
                        actions.push(action)
                    } else {
                        map.insert(
                            self.context_menu_id.unwrap(),
                            vec![action],
                        );
                    }
                } else {
                    let mut map = HashMap::new();
                    map.insert(
                        self.context_menu_id.unwrap(),
                        vec![action],
                    );
                    self.slide_action_map = Some(map);
                }
            }
            Message::ActivateSlide(item_index, slide_index) => {
                debug!(slide_index, item_index);
                if let Some(slide) = self
                    .service
                    .get(item_index)
                    .and_then(|item| item.slides.get(slide_index))
                {
                    self.current_item = item_index;
                    self.current_slide_index = slide_index;
                    return self
                        .update(Message::SlideChange(slide.clone()));
                }
            }
            Message::SlideChange(slide) => {
                let slide_text = slide.text();
                debug!(slide_text, "slide changed");
                let bg = slide.background().clone();
                debug!(?bg, "comparing background...");
                let backgrounds_match =
                    self.current_slide.background()
                        == slide.background();
                // self.current_slide_index = slide;
                debug!("cloning slide...");
                self.current_slide = slide.clone();
                let _ =
                    self.update(Message::ChangeFont(slide.font()));
                debug!("changing video now...");
                if !backgrounds_match {
                    if let Some(video) = &mut self.video {
                        let _ = video.restart_stream();
                    }
                    self.reset_video();
                }

                let mut target_item = 0;

                self.service.iter().enumerate().try_for_each(
                    |(index, item)| {
                        item.slides.iter().enumerate().try_for_each(
                            |(slide_index, _)| {
                                target_item += 1;
                                if (index, slide_index)
                                    == (
                                        self.current_item,
                                        self.current_slide_index,
                                    )
                                {
                                    None
                                } else {
                                    Some(())
                                }
                            },
                        )
                    },
                );

                debug!(target_item);

                let offset = AbsoluteOffset {
                    x: {
                        if target_item > 2 {
                            (target_item as f32)
                                .mul_add(187.5, -187.5)
                        } else {
                            0.0
                        }
                    },
                    y: 0.0,
                };

                let mut tasks = vec![];
                tasks.push(scroll_to(self.scroll_id.clone(), offset));

                if self.slide_action_map.is_some() {
                    debug!("Found slide actions, running them");
                    tasks.push(self.run_slide_actions());
                };

                if let Some(mut new_audio) =
                    self.current_slide.audio()
                {
                    if let Some(stripped_audio) = new_audio
                        .to_str()
                        .unwrap()
                        .to_string()
                        .strip_prefix(r"file://")
                    {
                        new_audio = PathBuf::from(stripped_audio);
                    }
                    debug!("{:?}", new_audio);
                    if new_audio.exists() {
                        let old_audio = self.audio.clone();
                        match old_audio {
                            Some(current_audio)
                                if current_audio != *new_audio =>
                            {
                                debug!(
                                    ?new_audio,
                                    ?current_audio,
                                    "audio needs to change"
                                );
                                self.audio = Some(new_audio);
                                tasks.push(self.start_audio());
                            }
                            Some(current_audio) => {
                                debug!(
                                    ?new_audio,
                                    ?current_audio,
                                    "Same audio shouldn't change"
                                );
                            }
                            None => {
                                debug!(
                                    ?new_audio,
                                    "could not find audio, need to change"
                                );
                                self.audio = Some(new_audio);
                                tasks.push(self.start_audio());
                            }
                        }
                    } else {
                        self.audio = None;
                        self.update(Message::EndAudio);
                    }
                } else {
                    self.audio = None;
                    self.update(Message::EndAudio);
                }
                let task_count = tasks.len();
                debug!(?task_count);
                return Action::Task(Task::batch(tasks));
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
                if let Some(video) = &mut self.video {
                    video.set_paused(false);
                    video
                        .set_looping(self.current_slide.video_loop());
                }
            }
            Message::VideoPos(position) => {
                if let Some(video) = &mut self.video {
                    let position = Position::Time(
                        std::time::Duration::from_secs_f32(position),
                    );
                    match video.seek(position, false) {
                        Ok(()) => debug!(
                            "Video position changed: {:?}",
                            position
                        ),
                        Err(e) => error!(
                            "Problem changing video position: {e}"
                        ),
                    }
                }
            }
            Message::VideoFrame => {
                if let Some(video) = &self.video {
                    self.video_position =
                        video.position().as_secs_f32();
                }
            }
            Message::MissingPlugin(element) => {
                if let Some(video) = &mut self.video {
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
                                        gstreamer::Registry::update()
                                    );
                                }
                                Err(err) => {
                                    warn!("failed to parse missing plugin message: {err}");
                                }
                            }
                            Message::None
                        })
                        .await
                        .unwrap()
                    },
                    |x| x,
                ));
            }
            Message::HoveredSlide(slide) => {
                self.hovered_slide = slide;
            }
            Message::StartAudio => {
                return Action::Task(self.start_audio());
            }
            Message::EndAudio => {
                self.sink.0.stop();
            }
            Message::None => debug!("Presenter Message::None"),
            Message::Error(error) => {
                error!(error);
            }
        }
        Action::None
    }

    pub fn view(&self) -> Element<Message> {
        slide_view(&self.current_slide, &self.video, false, true)
    }

    pub fn view_preview(&self) -> Element<Message> {
        slide_view(&self.current_slide, &self.video, false, false)
    }

    pub fn preview_bar(&self) -> Element<Message> {
        let mut items = vec![];
        self.service.iter().enumerate().for_each(
            |(item_index, item)| {
                let mut slides = vec![];
                item.slides.iter().enumerate().for_each(
                    |(slide_index, slide)| {
                        let is_current_slide =
                            (item_index, slide_index)
                                == (
                                    self.current_item,
                                    self.current_slide_index,
                                );

                        let container = slide_view(
                            slide,
                            &self.video,
                            true,
                            false,
                        );
                        let delegate = mouse_area(
                            Container::new(container)
                                .style(move |t| {
                                    let mut style =
                                        container::Style::default();
                                    let theme = t.cosmic();
                                    let hovered = self.hovered_slide
                                        == Some((
                                            item_index,
                                            slide_index,
                                        ));
                                    style.background =
                                        Some(Background::Color(
                                            if is_current_slide {
                                                theme
                                                    .accent
                                                    .base
                                                    .into()
                                            } else if hovered {
                                                theme
                                                    .accent
                                                    .hover
                                                    .into()
                                            } else {
                                                theme
                                                    .palette
                                                    .neutral_3
                                                    .into()
                                            },
                                        ));
                                    style.border = Border::default()
                                        .rounded(10.0);
                                    style.shadow = Shadow {
                                        color: Color::BLACK,
                                        offset: {
                                            if is_current_slide {
                                                Vector::new(5.0, 5.0)
                                            } else if hovered {
                                                Vector::new(5.0, 5.0)
                                            } else {
                                                Vector::new(0.0, 0.0)
                                            }
                                        },
                                        blur_radius: {
                                            if is_current_slide {
                                                10.0
                                            } else if hovered {
                                                10.0
                                            } else {
                                                0.0
                                            }
                                        },
                                    };
                                    style
                                })
                                .center_x(100.0 * 16.0 / 9.0)
                                .height(100)
                                .padding(10),
                        )
                        .interaction(
                            cosmic::iced::mouse::Interaction::Pointer,
                        )
                        .on_move(move |_| {
                            Message::HoveredSlide(Some((
                                item_index,
                                slide_index,
                            )))
                        })
                        .on_exit(Message::HoveredSlide(None))
                        .on_release(Message::ClickSlide(
                            item_index,
                            slide_index,
                        ))
                        .on_right_release(Message::RightClickSlide(
                            item_index,
                            slide_index,
                        ));
                        slides.push(delegate.into());
                    },
                );
                let row = Row::from_vec(slides)
                    .spacing(10)
                    .padding([20, 15]);
                let label = text::body(item.title.clone());
                let label_container = container(label)
                    .align_top(Length::Fill)
                    .align_left(Length::Fill)
                    .padding([0, 0, 0, 35]);
                let divider = vertical_rule(2);
                items.push(
                    container(stack!(row, label_container))
                        .padding([5, 2])
                        .into(),
                );
                items.push(divider.into());
            },
        );
        let scrollable =
            scrollable(container(Row::from_vec(items)).style(|_t| {
                let style = container::Style::default();
                style.border(Border::default().width(2))
            }))
            .direction(Direction::Horizontal(Scrollbar::new()))
            .height(Length::Fill)
            .width(Length::Fill)
            .id(self.scroll_id.clone());
        self.context_menu(scrollable.into())
    }

    fn context_menu<'a>(
        &self,
        items: Element<'a, Message>,
    ) -> Element<'a, Message> {
        if self.context_menu_id.is_some() {
            let mut scenes = vec![];
            if let Some(obs_scenes) = &self.obs_scenes {
                for scene in obs_scenes {
                    let menu_item = menu::Item::Button(
                        scene.id.name.clone(),
                        None,
                        MenuAction::ObsSceneAssign(scene.index),
                    );
                    scenes.push(menu_item);
                }
            }
            let menu_items = vec![
                menu::Item::Button(
                    "Start Stream".to_string(),
                    None,
                    MenuAction::ObsStartStream,
                ),
                menu::Item::Button(
                    "Stop Stream".to_string(),
                    None,
                    MenuAction::ObsStopStream,
                ),
                menu::Item::Folder("Obs Scene".to_string(), scenes),
            ];
            let context_menu = context_menu(
                items,
                self.context_menu_id.map_or_else(
                    || None,
                    |_| {
                        Some(menu::items(&HashMap::new(), menu_items))
                    },
                ),
            );
            Element::from(context_menu)
        } else {
            items
        }
    }

    // fn slide_delegate(&self, slide: &Slide) -> Element<'_, Message> {
    //     let font_name = slide.font().into_boxed_str();
    //     let family = Family::Name(Box::leak(font_name));
    //     let weight = Weight::Normal;
    //     let stretch = Stretch::Normal;
    //     let style = Style::Normal;
    //     let font = Font {
    //         family,
    //         weight,
    //         stretch,
    //         style,
    //     };

    //     let is_current_slide = self.service[self.current_item]
    //         .slides
    //         .get(self.current_slide_index as usize)
    //         .is_some();
    //     let this_item_index = self
    //         .service
    //         .iter()
    //         .position(|item| item == self.service[self.current_item])
    //         .unwrap();
    //     let this_slide_index = self.service[this_item_index].slides;

    //     let container =
    //         slide_view(slide.clone(), &self.video, font, true, false);
    //     let delegate = mouse_area(
    //         Container::new(container)
    //             .style(move |t| {
    //                 let mut style = container::Style::default();
    //                 let theme = t.cosmic();
    //                 let hovered = self.hovered_slide == slide_id;
    //                 style.background = Some(Background::Color(
    //                     if is_current_slide {
    //                         theme.accent.base.into()
    //                     } else if hovered {
    //                         theme.accent.hover.into()
    //                     } else {
    //                         theme.palette.neutral_3.into()
    //                     },
    //                 ));
    //                 style.border = Border::default().rounded(10.0);
    //                 style.shadow = Shadow {
    //                     color: Color::BLACK,
    //                     offset: {
    //                         if is_current_slide {
    //                             Vector::new(5.0, 5.0)
    //                         } else if hovered {
    //                             Vector::new(5.0, 5.0)
    //                         } else {
    //                             Vector::new(0.0, 0.0)
    //                         }
    //                     },
    //                     blur_radius: {
    //                         if is_current_slide {
    //                             10.0
    //                         } else if hovered {
    //                             10.0
    //                         } else {
    //                             0.0
    //                         }
    //                     },
    //                 };
    //                 style
    //             })
    //             .center_x(100.0 * 16.0 / 9.0)
    //             .height(100)
    //             .padding(10),
    //     )
    //     .interaction(cosmic::iced::mouse::Interaction::Pointer)
    //     .on_move(move |_| Message::HoveredSlide(slide_id))
    //     .on_exit(Message::HoveredSlide(-1))
    //     .on_press(Message::SlideChange(slide.clone()));
    //     delegate.into()
    // }

    fn reset_video(&mut self) {
        match self.current_slide.background().kind {
            BackgroundKind::Video => {
                let path = &self.current_slide.background().path;
                if path.exists() {
                    let url = Url::from_file_path(path).unwrap();
                    let result = Self::create_video(url);
                    match result {
                        Ok(mut v) => {
                            v.set_looping(
                                self.current_slide.video_loop(),
                            );
                            self.video = Some(v);
                        }
                        Err(e) => {
                            error!(
                                "Had an error creating the video object: {e}"
                            );
                            self.video = None;
                        }
                    }
                } else {
                    self.video = None;
                }
            }
            _ => self.video = None,
        }
    }

    fn start_audio(&mut self) -> Task<Message> {
        if let Some(audio) = &mut self.audio {
            debug!(?audio, "This is where audio should be changing");
            let audio = audio.clone();
            Task::perform(
                start_audio(Arc::clone(&self.sink.0), audio),
                |()| Message::None,
            )
        } else {
            debug!(?self.audio, "Apparently this doesn't exist");
            Task::none()
        }
    }

    pub fn update_items(&mut self, items: Vec<ServiceItem>) {
        let total_slides: usize =
            items.iter().fold(0, |a, item| a + item.slides.len());
        self.service = items;
        self.total_slides = total_slides;
    }

    pub fn run_slide_actions(&self) -> Task<Message> {
        let mut tasks = vec![];
        let item_index = self.current_item;
        let slide_index = self.current_slide_index;

        if let Some(map) = &self.slide_action_map {
            if let Some(actions) = map.get(&(item_index, slide_index))
            {
                for action in actions {
                    match action {
                        slide_actions::Action::Obs { action } => {
                            debug!("found obs slide actions");
                            if let Some(obs) = &self.obs_client {
                                let obs = Arc::clone(&obs);
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
        }
        Task::batch(tasks)
    }
}

#[allow(clippy::unused_async)]
async fn obs_scene_switch(client: Arc<Client>, scene: Scene) {
    match client.scenes().set_current_program_scene(&scene.id).await {
        Ok(_) => debug!("Set scene to: {:?}", scene),
        Err(e) => error!(?e),
    }
}

// This needs to be async so that rodio's audio will work
#[allow(clippy::unused_async)]
async fn start_audio(sink: Arc<Sink>, audio: PathBuf) {
    debug!(?audio);
    let file = BufReader::new(File::open(audio).unwrap());
    debug!(?file);
    let source = Decoder::new(file).unwrap();
    sink.append(source);
    let empty = sink.empty();
    let paused = sink.is_paused();
    debug!(empty, paused, "Finished running");
}

fn scale_font(font_size: f32, width: f32) -> f32 {
    let scale_factor = (REFERENCE_WIDTH / width).sqrt();
    // debug!(scale_factor);

    if font_size > 0.0 {
        font_size / scale_factor
    } else {
        50.0
    }
}

pub(crate) fn slide_view<'a>(
    slide: &'a Slide,
    video: &'a Option<Video>,
    delegate: bool,
    hide_mouse: bool,
) -> Element<'a, Message> {
    let res = responsive(move |size| {
        let width = size.height * 16.0 / 9.0;

        let text: Element<Message> =
            if let Some(text) = &slide.text_svg {
                if let Some(handle) = &text.handle {
                    image(handle)
                        .content_fit(ContentFit::ScaleDown)
                        .width(Length::Shrink)
                        .height(Length::Shrink)
                        .into()
                } else {
                    Space::with_width(0).into()
                }
            } else {
                Space::with_width(0).into()
            };
        let black = Container::new(Space::new(0, 0))
            .style(|_| {
                container::background(Background::Color(Color::BLACK))
            })
            .clip(true)
            .width(width)
            .height(size.height);
        let background = match slide.background().kind {
            BackgroundKind::Image => {
                let path = slide.background().path.clone();
                Container::new(
                    image(path)
                        .content_fit(ContentFit::Cover)
                        .width(width)
                        .height(size.height),
                )
                .center(Length::Shrink)
                .clip(true)
            }
            BackgroundKind::Video => {
                if delegate {
                    Container::new(Space::new(0, 0))
                        .style(|_| {
                            container::background(Background::Color(
                                Color::BLACK,
                            ))
                        })
                        .center(Length::Fill)
                        .clip(true)
                        .width(width)
                        .height(size.height)
                } else if let Some(video) = &video {
                    Container::new(
                        VideoPlayer::new(video)
                            .mouse_hidden(hide_mouse)
                            .width(width)
                            .height(size.height)
                            .on_end_of_stream(Message::EndVideo)
                            .on_new_frame(Message::VideoFrame)
                            .on_missing_plugin(Message::MissingPlugin)
                            .on_warning(|w| {
                                Message::Error(w.to_string())
                            })
                            .on_error(|e| {
                                Message::Error(e.to_string())
                            })
                            .content_fit(ContentFit::Cover),
                    )
                    .center(Length::Fill)
                    .clip(true)
                    // Container::new(Space::new(0, 0))
                } else {
                    Container::new(Space::new(0, 0))
                }
            }
            BackgroundKind::Pdf => {
                if let Some(pdf) = slide.pdf_page() {
                    Container::new(
                        image(pdf).content_fit(ContentFit::Contain),
                    )
                    .center(Length::Fill)
                    .clip(true)
                } else {
                    Container::new(Space::new(0.0, 0.0))
                        .center(Length::Fill)
                        .clip(true)
                }
            }
            BackgroundKind::Html => todo!(),
        };
        let stack = stack!(
            black,
            background.center_x(Length::Fill),
            container(text).center(Length::Fill)
        );
        Container::new(stack).center(Length::Fill).into()
    });

    res.into()
}
