use core::fmt;
use std::error::Error;
use std::time::Duration;

use obws::responses::scenes::Scenes;
use obws::Client;
use tracing::debug;

pub struct Obs {
    scenes: Scenes,
    client: Option<Client>,
}

impl fmt::Debug for Obs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("host", &"localhost")
            .field("port", &4455)
            .finish()
    }
}

impl Clone for Obs {
    fn clone(&self) -> Self {
        Self {
            scenes: self.scenes.clone(),
            client: Some(make_client()),
        }
    }
}

impl Default for Obs {
    fn default() -> Self {
        Self {
            scenes: Scenes::default(),
            client: None,
        }
    }
}

impl Obs {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let client =
            Client::connect("localhost", 4455, Some("")).await?;
        let scene_list = client.scenes().list().await?;
        debug!(?scene_list);
        Ok(Self {
            scenes: scene_list,
            client: Some(client),
        })
    }

    pub fn get_list(self) -> Result<Vec<String>, Box<dyn Error>> {
        let scenes = self
            .scenes
            .scenes
            .iter()
            .map(|x| x.name.clone())
            .collect::<Vec<String>>();
        if scenes.len() > 0 {
            Ok(scenes)
        } else {
            Err(format!("Scenes found: {}", scenes.len()))?
        }
    }

    pub fn set_scene(
        self,
        scene: String,
    ) -> Result<(), Box<dyn Error>> {
        debug!("Starting function");
        if self.client.is_some() {
            debug!("Starting to set");
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async move {
                debug!(scene, "working in thread");
                self.client
                    .unwrap()
                    .scenes()
                    .set_current_program_scene(scene.as_str()).await
            })?;
            Ok(())
        } else {
            Err("There is no client".to_owned())?
        }
    }
}

fn make_client() -> Client {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let future = Client::connect("localhost", 4455, Some(""));
    let client = runtime.block_on(future).unwrap();
    client
}

#[cxx_qt::bridge]
mod obs_model {
    use tracing::{debug, error};

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qlist.h");
        type QList_QString = cxx_qt_lib::QList<QString>;
    }

    #[cxx_qt::qobject]
    #[derive(Debug, Default)]
    pub struct ObsModel {
        #[qproperty]
        scenes: QStringList,
        #[qproperty]
        port: QString,
        #[qproperty]
        connected: bool,
        obs: Option<super::Obs>,
    }


    impl qobject::ObsModel {
        #[qinvokable]
        pub fn update_scenes(mut self: Pin<&mut Self>) -> QStringList {
            debug!("updating scenes");
            let mut scenes_list = QList_QString::default();
            if let Some(obs) = self.obs() {
                debug!("found obs");
                for scene in obs.scenes.scenes.iter().rev() {
                    debug!(?scene);
                    scenes_list.append(QString::from(&scene.name));
                }
            }
            for s in scenes_list.iter() {
                debug!(?s);
            }
            let list = QStringList::from(&scenes_list);
            debug!(?list);
            self.as_mut().set_scenes(list.clone());
            list
        }

        #[qinvokable]
        pub fn get_obs(mut self: Pin<&mut Self>) -> bool {
            debug!("getting obs");

            tokio::runtime::Runtime::new().unwrap().block_on(async {
                match super::Obs::new().await {
                    Ok(o) => {
                        self.as_mut().set_connected(true);
                        self.as_mut().set_obs(Some(o));
                        self.as_mut().update_scenes();
                    },
                    Err(e) => {
                        error!(e);
                        self.as_mut().set_connected(false);
                    },
                }
            });

            if let Some(_obs) = self.obs() {
                true
            } else {
                false
            }
        }

        #[qinvokable]
        pub fn set_scene(mut self: Pin<&mut Self>, scene: QString) {
            let scene = scene.to_string();
            if let Some(obs) = self.obs_mut() {
                let obs = obs.clone();
                match obs.set_scene(scene) {
                    Ok(()) => debug!("Successfully set scene"),
                    Err(e) => error!(e),
                }
            }
        }
    }
}
