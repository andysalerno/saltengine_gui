use super::{
    bi_channel::BiChannel,
    gui_notifier::GuiNotifier,
    messages::{FromGui, ToGui},
};
use async_trait::async_trait;
use log::info;
use salt_engine::{
    game_logic::events::{ClientActionEvent, EndTurnEvent, SummonCreatureFromHandEvent},
    game_runner::GameClient,
    game_state::{
        board::{BoardPos, RowId},
        PlayerId,
    },
};

pub(crate) struct GuiClient {
    player_id: PlayerId,
    channel: BiChannel<ToGui, FromGui>,
}

impl GuiClient {
    pub fn new_with_id(channel: BiChannel<ToGui, FromGui>, player_id: PlayerId) -> Self {
        channel.send_blocking(ToGui::PlayerIdSet(player_id));
        Self { player_id, channel }
    }

    fn id(&self) -> salt_engine::game_state::PlayerId {
        self.player_id
    }
}

#[async_trait]
impl GameClient for GuiClient {
    async fn next_action(
        &mut self,
        _game_state: salt_engine::game_state::GameStatePlayerView,
    ) -> ClientActionEvent {
        info!("next_action invoked on GuiClient. Waiting for message from godot...");

        match self.channel.recv().await.unwrap() {
            FromGui::SummonFromHandToSlotRequest {
                slot_path,
                card_instance_id,
            } => ClientActionEvent::SummonCreatureFromHand(SummonCreatureFromHandEvent::new(
                self.id(),
                BoardPos::new(self.id(), RowId::BackRow, 0),
                card_instance_id,
            )),
            FromGui::EndTurnAction => ClientActionEvent::EndTurn(EndTurnEvent),
        }
    }

    async fn make_prompter(&self) -> Box<dyn salt_engine::game_agent::Prompter> {
        todo!()
    }

    async fn observe_state_update(
        &mut self,
        game_state: salt_engine::game_state::GameStatePlayerView,
    ) {
        info!("GuiClient::observe_state_update()");
        let message = ToGui::StateUpdate(game_state);
        self.channel.send_blocking(message).unwrap();
    }

    async fn make_notifier(&self) -> Box<dyn salt_engine::game_agent::ClientNotifier> {
        Box::new(GuiNotifier::new(self.channel.clone()))
    }

    async fn on_turn_start(&mut self, _game_state: &salt_engine::game_state::GameState) {
        info!("GuiClient saw: on_turn_start()");
    }
}
