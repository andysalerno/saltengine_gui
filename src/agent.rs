use salt_engine::{
    game_agent::game_agent::GameAgent,
    game_logic::{ClientGameEvent, EndTurnEvent},
    game_state::PlayerId,
};

pub(crate) struct GuiAgent {
    player_id: PlayerId,
}

impl GuiAgent {
    pub fn new_with_id(player_id: PlayerId) -> Self {
        Self { player_id }
    }
}

impl GameAgent for GuiAgent {
    fn get_action(
        &self,
        game_state: &salt_engine::game_state::GameStatePlayerView,
    ) -> salt_engine::game_logic::ClientGameEvent {
        ClientGameEvent::EndTurn(EndTurnEvent)
    }

    fn id(&self) -> salt_engine::game_state::PlayerId {
        todo!()
    }

    fn make_prompter(&self) -> Box<dyn salt_engine::game_agent::game_agent::Prompter> {
        todo!()
    }
}
