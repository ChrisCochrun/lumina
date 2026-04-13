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
    cosmic_theme::Spacing,
    iced::{
        Background, Border, Color, ContentFit, Font, Length, Shadow,
        Vector,
        alignment::Horizontal,
        core::text::{Ellipsize, EllipsizeHeightLimit},
        font::{Family, Stretch, Style, Weight},
        widget::{
            grid,
            scrollable::{
                AbsoluteOffset, Direction, Scrollbar, scroll_to,
            },
            stack,
        },
    },
    prelude::*,
    theme,
    widget::{
        Container, Id, Row, Space, column, container, context_menu,
        divider::vertical, flex_row, image, menu, mouse_area,
        responsive, scrollable, space, text,
    },
};
use derive_more::Debug;
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
    ui::{gst_video, library::elide_text},
};

// const REFERENCE_WIDTH: f32 = 1920.0;
static DEFAULT_SLIDE: LazyLock<Slide> = LazyLock::new(Slide::default);

// #[derive(Default, Clone, Debug)]
pub(crate) struct Presenter {
    pub service: Arc<Vec<ServiceItem>>,
    pub current_slide: Slide,
    pub current_item: usize,
    pub current_slide_index: usize,
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

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub(crate) enum Message {
    NextSlide,
    PrevSlide,
    SlideChange(Slide),
    ActivateSlide(usize, usize),
    ClickSlide(usize, usize),
    EndVideo,
    StartVideo,
    // StartAudio,
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
    #[debug("AddObsClient")]
    AddObsClient(Arc<Client>),
    AssignSlideAction(slide_actions::Action),
    PlayPauseVideo,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuAction {
    ObsSceneAssign(usize),
    ObsStartStream,
    ObsStopStream,
    // ObsStartRecord,
    // ObsStopRecord,
}

impl menu::Action for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            Self::ObsSceneAssign(scene) => {
                Message::AssignObsScene(*scene)
            }
            Self::ObsStartStream => Message::AssignSlideAction(
                slide_actions::Action::Obs {
                    action: ObsAction::StartStream,
                },
            ),
            Self::ObsStopStream => Message::AssignSlideAction(
                slide_actions::Action::Obs {
                    action: ObsAction::StopStream,
                },
            ),
            // Self::ObsStartRecord => todo!(),
            // Self::ObsStopRecord => todo!(),
        }
    }
}

impl Presenter {
    pub fn with_items(items: Arc<Vec<ServiceItem>>) -> Self {
        let video = {
            items.first().and_then(|item| {
                item.slides.first().and_then(|slide| {
                    let path = slide.background().path.clone();
                    if !path.exists() {
                        return None;
                    }
                    let url = Url::from_file_path(path).expect(
                        "There should be a video file here",
                    );
                    match Video::new(&url) {
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
        let total_slides: usize =
            items.iter().fold(0, |a, item| a + item.slides.len());

        let slide =
            items.first().and_then(|item| item.slides.first());
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
            current_item: 0,
            current_slide_index: 0,
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

    #[allow(clippy::too_many_lines)]
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::AddObsClient(client) => {
                self.obs_client = Some(client);
            }
            Message::NextSlide => {
                if self.service.get(self.current_item).is_some_and(
                    |i| {
                        i.slides.len() == self.current_slide_index + 1
                    },
                ) {
                    return self.update(Message::ActivateSlide(
                        self.current_item + 1,
                        0,
                    ));
                } else if self
                    .service
                    .get(self.current_item)
                    .is_some_and(|i| {
                        i.slides.len() > self.current_slide_index + 1
                    })
                {
                    return self.update(Message::ActivateSlide(
                        self.current_item,
                        self.current_slide_index + 1,
                    ));
                } else {
                    return Action::None;
                }
                // return Action::NextSlide;
            }
            Message::PrevSlide => {
                if self.current_item == 0
                    && self.current_slide_index == 0
                {
                    return Action::None;
                } else if self.current_slide_index == 0 {
                    let target_item = self.current_item - 1;
                    let last_slide = self.service.get(target_item).map(|i| i.slides.len() - 1).expect("We have checked that this item should be here.");
                    return self.update(Message::ActivateSlide(
                        target_item,
                        last_slide,
                    ));
                } else {
                    let target_slide = self.current_slide_index - 1;
                    return self.update(Message::ActivateSlide(
                        self.current_item,
                        target_slide,
                    ));
                }
                // return Action::PrevSlide;
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
                let slide_id = self.context_menu_id.expect("In this match we should always already have a context menu id");
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
                        }
                        *actions = altered_actions;
                        debug!(
                            "updating the obs scene {:?}",
                            new_scene
                        );
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
                        debug!(
                            "adding the obs scene {:?}",
                            new_scene
                        );
                    } else {
                        debug!(
                            "updating the obs scene {:?}",
                            new_scene
                        );
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
                let slide_id = self.context_menu_id.expect("In this match we should always already have a context menu id");
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
                let font = slide
                    .font()
                    .map_or_else(String::new, |font| font.get_name());
                let _ = self.update(Message::ChangeFont(font));
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

                #[allow(clippy::cast_precision_loss)]
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
                tasks.push(scroll_to(
                    self.scroll_id.clone(),
                    offset.into(),
                ));

                if self.slide_action_map.is_some() {
                    debug!("Found slide actions, running them");
                    tasks.push(self.run_slide_actions());
                }

                if let Some(mut new_audio) =
                    self.current_slide.audio()
                {
                    if let Some(stripped_audio) = new_audio
                        .to_str()
                        .expect("Should be no problem")
                        .to_string()
                        .strip_prefix(r"file://")
                    {
                        new_audio = PathBuf::from(stripped_audio);
                    }
                    debug!("{:?}", new_audio);
                    if new_audio.exists() {
                        self.audio = Some(new_audio);
                        tasks.push(self.start_audio());
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
            Message::PlayPauseVideo => {
                if let Some(video) = &mut self.video {
                    video.set_paused(!video.paused());
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
                    if self.video_position > 0.0
                        && video.position().as_secs_f32() != 0.0
                    {
                        self.video_position =
                            video.position().as_secs_f32();
                    }
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
                        .expect("Spawning a task shouldn't fail")
                    },
                    |x| x,
                ));
            }
            Message::HoveredSlide(slide) => {
                self.hovered_slide = slide;
            }
            // Message::StartAudio => {
            //     return Action::Task(self.start_audio());
            // }
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
        slide_view(
            &self.current_slide,
            self.video.as_ref(),
            false,
            true,
        )
    }

    pub fn view_preview(&self) -> Element<Message> {
        slide_view(
            &self.current_slide,
            self.video.as_ref(),
            false,
            false,
        )
    }

    #[allow(clippy::too_many_lines)]
    pub fn preview_grid(&self) -> Element<Message> {
        let Spacing {
            space_none,
            space_xs,
            space_s,
            space_m,
            space_l,
            ..
        } = theme::spacing();
        // let mut grid = grid(vec![]).spacing(space_m);
        let mut items = vec![];
        for (item_index, item) in self.service.iter().enumerate() {
            let slides_length = item.slides.len();
            for (slide_index, slide) in item.slides.iter().enumerate()
            {
                let is_current_slide = (item_index, slide_index)
                    == (self.current_item, self.current_slide_index);

                let slide = slide_view(
                    slide,
                    self.video.as_ref(),
                    true,
                    false,
                );
                let delegate = mouse_area(
                    Container::new(slide)
                        .style(move |t| {
                            let mut style =
                                container::Style::default();
                            let theme = t.cosmic();
                            let hovered = self.hovered_slide
                                == Some((item_index, slide_index));
                            style.background =
                                Some(Background::Color(
                                    if is_current_slide {
                                        theme.accent.base.into()
                                    } else if hovered {
                                        theme.accent.hover.into()
                                    } else {
                                        theme.palette.neutral_3.into()
                                    },
                                ));
                            style.border =
                                Border::default().rounded(10.0);
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
                .on_right_release(
                    Message::RightClickSlide(item_index, slide_index),
                );
                let item = if slide_index == 0 {
                    let label =
                        text::body(elide_text(&item.title, 150.0));

                    column![label, delegate]
                        .align_x(Horizontal::Center)
                        .spacing(space_s)
                        .apply(container)
                } else {
                    delegate.apply(container).padding([
                        space_l, space_none, space_none, space_none,
                    ])
                };
                items.push(item.into());

                items.push(
                    container(
                        vertical::light()
                            .width(20)
                            .height(Length::Fill),
                    )
                    .class(if slide_index + 1 == slides_length {
                        theme::Container::Card
                    } else {
                        theme::Container::WindowBackground
                    })
                    .padding([
                        space_none, space_xs, space_none, space_xs,
                    ])
                    .into(),
                );
            }
        }
        let scrollable = scrollable(
            container(
                flex_row(items)
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
        // .direction(Direction::Horizontal(Scrollbar::new()))
        .height(Length::Fill)
        .width(Length::Fill)
        .id(self.scroll_id.clone());
        self.context_menu(scrollable.into())
    }

    #[allow(clippy::too_many_lines)]
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
                            self.video.as_ref(),
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
                                            if is_current_slide
                                                || hovered
                                            {
                                                Vector::new(5.0, 5.0)
                                            } else {
                                                Vector::new(0.0, 0.0)
                                            }
                                        },
                                        blur_radius: {
                                            if is_current_slide
                                                || hovered
                                            {
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
                let divider = vertical::light();
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
                container(items),
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
                    let url = Url::from_file_path(path)
                        .expect("There should be a video file here");
                    let result = gst_video::create_video(&url, 60);
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

    pub fn update_items(&mut self, items: Arc<Vec<ServiceItem>>) {
        let total_slides: usize =
            items.iter().fold(0, |a, item| a + item.slides.len());
        self.service = items;
        self.total_slides = total_slides;
    }

    pub fn run_slide_actions(&self) -> Task<Message> {
        let mut tasks = vec![];
        let item_index = self.current_item;
        let slide_index = self.current_slide_index;

        if let Some(map) = &self.slide_action_map
            && let Some(actions) = map.get(&(item_index, slide_index))
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
}

// #[allow(clippy::unused_async)]
// async fn obs_scene_switch(client: Arc<Client>, scene: Scene) {
//     match client.scenes().set_current_program_scene(&scene.id).await {
//         Ok(()) => debug!("Set scene to: {:?}", scene),
//         Err(e) => error!(?e),
//     }
// }

// This needs to be async so that rodio's audio will work
#[allow(clippy::unused_async)]
async fn start_audio(sink: Arc<Sink>, audio: PathBuf) {
    debug!(?audio);
    let file = BufReader::new(
        File::open(audio)
            .expect("There should be an audio file here"),
    );
    debug!(?file);
    let source = Decoder::new(file)
        .expect("There should be an audio decoder here");
    sink.append(source);
    let empty = sink.empty();
    let paused = sink.is_paused();
    debug!(empty, paused, "Finished running");
}

pub(crate) fn slide_view<'a>(
    slide: &'a Slide,
    video: Option<&'a Video>,
    delegate: bool,
    hide_mouse: bool,
) -> Element<'a, Message> {
    responsive(move |size| {
        let width = size.height * 16.0 / 9.0;
        let black = Container::new(Space::new())
            .style(|_| {
                container::background(Background::Color(Color::BLACK))
            })
            .clip(true)
            .width(width)
            .height(Length::Fill);
        let mut stack = stack(vec![black.into()]);

        match slide.background().kind {
            BackgroundKind::Image => {
                let path = slide.background().path.clone();
                stack = stack.push(
                    Container::new(
                        image(path)
                            .content_fit(ContentFit::Contain)
                            .width(width)
                            .height(Length::Fill),
                    )
                    .center(Length::Fill)
                    .clip(true),
                );
            }
            BackgroundKind::Video => {
                if let Some(video) = &video
                    && !delegate
                {
                    stack = stack.push(
                        Container::new(
                            VideoPlayer::new(video)
                                .mouse_hidden(hide_mouse)
                                .width(width)
                                .height(Length::Fill)
                                .on_end_of_stream(Message::EndVideo)
                                .on_new_frame(Message::VideoFrame)
                                .on_missing_plugin(
                                    Message::MissingPlugin,
                                )
                                .on_warning(|w| {
                                    Message::Error(w.to_string())
                                })
                                .on_error(|e| {
                                    Message::Error(e.to_string())
                                })
                                .content_fit(ContentFit::Contain),
                        )
                        .center(Length::Fill)
                        .clip(true),
                    );
                }
            }
            BackgroundKind::Pdf => {
                if let Some(pdf) = slide.pdf_page() {
                    stack = stack.push(
                        Container::new(
                            image(pdf)
                                .content_fit(ContentFit::Contain),
                        )
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
            stack = stack.push(
                image(handle)
                    .content_fit(ContentFit::Contain)
                    .width(Length::Fill)
                    .height(Length::Fill),
            );
        }
        Container::new(stack).center(Length::Fill).into()
    })
    .into()
}

#[cfg(test)]
mod test {
    use crate::core::{
        presentations::{PresKind, Presentation},
        slide::TextAlignment,
        songs::{Song, VerseName},
    };

    use super::*;
    use miette::Result;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_next_slide() -> Result<()> {
        let service = test_service();
        let mut presenter = Presenter::with_items(Arc::new(service));
        presenter.update(Message::NextSlide);
        dbg!(&presenter.service);
        assert_eq!(presenter.current_item, 1);
        assert_eq!(presenter.current_slide_index, 0);
        presenter.update(Message::NextSlide);
        assert_eq!(presenter.current_item, 1);
        assert_eq!(presenter.current_slide_index, 1);
        presenter.update(Message::NextSlide);
        assert_eq!(presenter.current_item, 1);
        assert_eq!(presenter.current_slide_index, 2);
        presenter.update(Message::PrevSlide);
        assert_eq!(presenter.current_item, 1);
        assert_eq!(presenter.current_slide_index, 1);
        presenter.update(Message::PrevSlide);
        assert_eq!(presenter.current_item, 1);
        assert_eq!(presenter.current_slide_index, 0);
        presenter.update(Message::PrevSlide);
        assert_eq!(presenter.current_item, 0);
        assert_eq!(presenter.current_slide_index, 0);

        Ok(())
    }

    fn test_service() -> Vec<ServiceItem> {
        let mut service = Vec::new();
        let song = test_song();
        let video = test_video("Christ Nutshell".into());
        let presentation = test_presentation();
        let mut video_item = ServiceItem::from(&video);
        video_item.slides = video_item.to_slides().unwrap();
        let mut song_item = ServiceItem::from(&song);
        song_item.slides = song_item.to_slides().unwrap();
        let mut pres_item = ServiceItem::from(&presentation);
        pres_item.slides = pres_item.to_slides().unwrap();
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
            ron::from_str(&lyrics).unwrap();
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
            background: Some(crate::core::slide::Background::try_from("file:///home/chris/nc/tfc/openlp/Flood/motions/Ocean_Floor_HD.mp4").unwrap()),
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
        }
    }
}
