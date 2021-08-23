use super::{
    bi_channel::BiChannel,
    gui_notifier::GuiNotifier,
    messages::{FromGui, ToGui},
};
use async_trait::async_trait;
use log::info;
use salt_engine::{
    game_logic::{ClientActionEvent, EndTurnEvent, SummonCreatureFromHandEvent},
    game_runner::GameClient,
    game_state::PlayerId,
};

pub(crate) struct GuiAgent {
    _player_id: PlayerId,
    channel: BiChannel<ToGui, FromGui>,
}

impl GuiAgent {
    pub fn new_with_id(channel: BiChannel<ToGui, FromGui>, player_id: PlayerId) -> Self {
        Self {
            _player_id: player_id,
            channel,
        }
    }

    fn id(&self) -> salt_engine::game_state::PlayerId {
        todo!()
    }
}

#[async_trait]
impl GameClient for GuiAgent {
    async fn next_action(
        &mut self,
        _game_state: salt_engine::game_state::GameStatePlayerView,
    ) -> ClientActionEvent {
        smol::block_on(async {
            let FromGui::SummonFromHandToSlotRequest(request) = self.channel.recv().await.unwrap();
            info!("Request FromGui to summon: {}", request);
        });

        ClientActionEvent::EndTurn(EndTurnEvent)
    }

    async fn make_prompter(&self) -> Box<dyn salt_engine::game_agent::Prompter> {
        todo!()
    }

    async fn observe_state_update(
        &mut self,
        game_state: salt_engine::game_state::GameStatePlayerView,
    ) {
        let message = ToGui::StateUpdate(game_state);
        self.channel.send_blocking(message).unwrap();
    }

    async fn make_notifier(&self) -> Box<dyn salt_engine::game_agent::ClientNotifier> {
        Box::new(GuiNotifier::new(self.channel.clone()))
    }

    async fn on_turn_start(&mut self, game_state: &salt_engine::game_state::GameState) {
        todo!()
    }
}
