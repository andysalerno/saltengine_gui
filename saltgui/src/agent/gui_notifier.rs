use async_trait::async_trait;
use log::info;
use salt_engine::{game_agent::ClientNotifier, game_logic::events::ClientEventView};

use super::{
    bi_channel::BiChannel,
    messages::{FromGui, ToGui},
};

/// A `ClientNotifier` implementation for use with the Godot gui.
pub(crate) struct GuiNotifier {
    channel: BiChannel<ToGui, FromGui>,
}

impl GuiNotifier {
    pub fn new(channel: BiChannel<ToGui, FromGui>) -> Self {
        Self { channel }
    }
}

#[async_trait]
impl ClientNotifier for GuiNotifier {
    async fn notify(&self, event: ClientEventView) {
        info!("GuiNotifier received an event: {:?}", event);
        let msg = ToGui::ClientEvent(event);
        self.channel.send(msg).await.expect("Failed to send");
    }
}
