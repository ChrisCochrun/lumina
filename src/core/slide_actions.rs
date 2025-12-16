use miette::{IntoDiagnostic, Result};
use std::sync::Arc;
use tracing::warn;

use obws::{Client, responses::scenes::Scene};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ObsAction {
    Scene { scene: Scene },
    StartStream,
    StopStream,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Action {
    Obs { action: ObsAction },
    Other,
}

impl ObsAction {
    pub async fn run(&self, client: Arc<Client>) -> Result<()> {
        match self {
            ObsAction::Scene { scene } => {
                warn!(?scene, "Changing obs scenes");
                client
                    .scenes()
                    .set_current_program_scene(&scene.id)
                    .await
                    .into_diagnostic()?;
            }
            ObsAction::StartStream => {
                client.streaming().start().await.into_diagnostic()?
            }
            ObsAction::StopStream => {
                client.streaming().stop().await.into_diagnostic()?
            }
        }
        Ok(())
    }
}
