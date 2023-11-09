use core::fmt;
use std::error::Error;

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
        if self.client.is_some() {
            if let Some(scene) = self
                .scenes
                .scenes
                .iter()
                .filter(|x| x.name == scene)
                .next()
            {
                self.client
                    .unwrap()
                    .scenes()
                    .set_current_program_scene(scene.name.as_str());
                Ok(())
            } else {
                Err("Couldn't set the scene".to_owned())?
            }
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
