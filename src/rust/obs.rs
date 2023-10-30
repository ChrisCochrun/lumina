use std::error::Error;

use obws::Client;
use obws::responses::scenes::Scenes;
use tracing::debug;

pub struct Obs {
    scenes: Scenes,
    client: Client,
}

impl Obs {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let client = Client::connect("localhost", 4455, Some("")).await?;
        let scene_list = client.scenes().list().await?;
        debug!(?scene_list);
        Ok(Self{scenes: scene_list, client})
    }

    pub fn get_list(self) -> Result<Vec<String>, Box<dyn Error>> {
        let scenes = self.scenes.scenes.iter().map(|x| x.name).collect::<Vec<String>>();
        if scenes.len() > 0 {
            Ok(scenes)
        } else {
            Err(format!("Scenes found: {}", scenes.len()))?
        }
    }

    pub fn set_scene(self, scene: String) -> Result<(), Box<dyn Error>> {
        if let Some(scene) = self.scenes.scenes.iter().filter(|x| x.name == scene).next() {
            self.client.scenes().set_current_program_scene(scene.name.as_str()); 
            Ok(())
        } else {
            Err("Couldn't set the scene".to_owned())?
        }
    }
}
