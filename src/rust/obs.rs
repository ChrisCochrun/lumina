use core::fmt;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QString, QStringList};
use obws::responses::scenes::Scenes;
use obws::Client;


use std::{error::Error, pin::Pin};
use tracing::{debug, error};

use crate::obs::obs::QList_QString;

#[derive(Default)]
pub struct Obs {
    scenes: Scenes,
    client: Option<Client>,
    current_program_scene: Option<String>,
}

impl fmt::Debug for Obs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("scenes", &self.scenes)
            .field("client", &self.client.is_some())
            .field("current_program_scene", &self.current_program_scene)
            .finish()
    }
}

impl Clone for Obs {
    fn clone(&self) -> Self {
        Self {
            scenes: self.scenes.clone(),
            client: Some(make_client()),
            current_program_scene: self.current_program_scene.clone(),
        }
    }
}



impl Obs {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let client =
            Client::connect("localhost", 4455, Some("")).await?;
        let scenes_object = client.scenes();
        let scene_list = scenes_object.list().await?;
        let current_program_scene =
            scenes_object.current_program_scene().await?;
        debug!(?scene_list);
        Ok(Self {
            scenes: scene_list,
            client: Some(client),
            current_program_scene: Some(current_program_scene),
        })
    }

    pub fn get_list(self) -> Result<Vec<String>, Box<dyn Error>> {
        let scenes = self
            .scenes
            .scenes
            .iter()
            .map(|x| x.name.clone())
            .collect::<Vec<String>>();
        if !scenes.is_empty() {
            Ok(scenes)
        } else {
            Err(format!("Scenes found: {}", scenes.len()))?
        }
    }

    pub fn set_scene(
        &self,
        scene: String,
    ) -> Result<(), Box<dyn Error>> {
        // debug!("Starting function");
        if let Some(_client) = &self.client {
            debug!(scene, "setting scene in obs");

            let client = make_client();
            let runtime = tokio::runtime::Runtime::new()?;
            let _handle = runtime.spawn(async move {
                debug!("in spawn: before setting");
                let res = client
                    .scenes()
                    .set_current_program_scene(&scene)
                    .await;
                match res {
                    Ok(_o) => {
                        debug!("in spawn: after setting: success")
                    }
                    Err(e) => error!(?e, "in spawn: after setting"),
                }
            });

            debug!("after spawn");

            // loop {
            //     sleep(Duration::from_millis(100));
            //     if handle.is_finished() {
            //         break;
            //     }
            // }

            Ok(())
        } else {
            Err("There is no client".to_owned())?
        }
    }
}

fn make_client() -> Client {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let future = Client::connect("localhost", 4455, Some(""));
    
    runtime.block_on(future).unwrap()
}

#[cxx_qt::bridge]
mod obs {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qlist.h");
        type QList_QString = cxx_qt_lib::QList<QString>;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QStringList, scenes)]
        #[qproperty(QString, port)]
        #[qproperty(bool, connected)]
        #[qproperty(QString, current_program_scene)]
        type ObsModel = super::ObsModelRust;

        #[qinvokable]
        fn update_scenes(self: Pin<&mut ObsModel>) -> QStringList;
        #[qinvokable]
        fn get_obs(self: Pin<&mut ObsModel>) -> bool;
        #[qinvokable]
        fn set_scene(self: Pin<&mut ObsModel>, scene: QString);
    }
}

#[derive(Debug, Default)]
pub struct ObsModelRust {
    scenes: QStringList,
    port: QString,
    connected: bool,
    obs: Option<Obs>,
    current_program_scene: QString,
}

impl obs::ObsModel {
    pub fn update_scenes(mut self: Pin<&mut Self>) -> QStringList {
        debug!("updating scenes");
        let mut scenes_list = QList_QString::default();
        if let Some(obs) = &self.as_mut().rust_mut().obs {
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

    pub fn get_obs(mut self: Pin<&mut Self>) -> bool {
        debug!("getting obs");

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            match Obs::new().await {
                Ok(o) => {
                    if let Some(scene) = &o.current_program_scene {
                        let scene = QString::from(scene);
                        self.as_mut()
                            .set_current_program_scene(scene);
                    }
                    self.as_mut().set_connected(true);
                    self.as_mut().rust_mut().obs = Some(o);
                    self.as_mut().update_scenes();
                }
                Err(e) => {
                    error!(e);
                    self.as_mut().set_connected(false);
                }
            }
        });

        if let Some(_obs) = &self.as_mut().rust_mut().obs {
            true
        } else {
            false
        }
    }

    pub fn set_scene(mut self: Pin<&mut Self>, scene: QString) {
        let scene = scene.to_string();
        if let Some(obs) = &self.as_mut().rust_mut().obs {
            // let obs = obs.clone();
            match obs.set_scene(scene) {
                Ok(()) => debug!("Successfully set scene"),
                Err(e) => error!(e),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_obs_setting_scene() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let future = Client::connect("localhost", 4455, Some(""));
        let client = runtime.block_on(future).unwrap();
        let scene = String::from("me");
        let res = runtime.block_on(
            client.scenes().set_current_program_scene(&scene),
        );
        debug_assert!(res.is_ok());
    }
}
