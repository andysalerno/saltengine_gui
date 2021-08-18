use async_trait::async_trait;
use salt_engine::{game_agent::game_agent::ClientNotifier, game_logic::ClientEventView};

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
        let msg = ToGui::ClientEvent(event);
        self.channel.send(msg).await.expect("Failed to send");
    }
}
