use super::{
    bi_channel::BiChannel,
    gui_notifier::GuiNotifier,
    messages::{FromGui, ToGui},
};
use log::info;
use salt_engine::{
    game_agent::game_agent::GameAgent,
    game_logic::{ClientActionEvent, EndTurnEvent, SummonCreatureFromHandEvent},
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
}

impl GameAgent for GuiAgent {
    fn get_action(
        &self,
        _game_state: &salt_engine::game_state::GameStatePlayerView,
    ) -> ClientActionEvent {
        smol::block_on(async {
            let FromGui::SummonFromHandToSlotRequest(request) = self.channel.recv().await.unwrap();
            info!("Request FromGui to summon: {}", request);
        });

        ClientActionEvent::EndTurn(EndTurnEvent)
    }

    fn id(&self) -> salt_engine::game_state::PlayerId {
        todo!()
    }

    fn make_prompter(&self) -> Box<dyn salt_engine::game_agent::game_agent::Prompter> {
        todo!()
    }

    fn observe_state_update(&self, game_state: salt_engine::game_state::GameStatePlayerView) {
        let message = ToGui::StateUpdate(game_state);
        self.channel.send_blocking(message).unwrap();
    }

    fn make_client_notifier(&self) -> Box<dyn salt_engine::game_agent::game_agent::ClientNotifier> {
        Box::new(GuiNotifier::new(self.channel.clone()))
    }
}
