use crossbeam::channel::Sender;
use salt_engine::{
    game_agent::game_agent::GameAgent,
    game_logic::{ClientGameEvent, EndTurnEvent},
    game_state::PlayerId,
};

use crate::gui_message::GuiMessage;

pub(crate) struct GuiAgent {
    player_id: PlayerId,
    sender: Sender<GuiMessage>,
}

impl GuiAgent {
    pub fn new_with_id(sender: Sender<GuiMessage>, player_id: PlayerId) -> Self {
        Self { sender, player_id }
    }
}

impl GameAgent for GuiAgent {
    fn get_action(
        &self,
        _game_state: &salt_engine::game_state::GameStatePlayerView,
    ) -> salt_engine::game_logic::ClientGameEvent {
        ClientGameEvent::EndTurn(EndTurnEvent)
    }

    fn id(&self) -> salt_engine::game_state::PlayerId {
        todo!()
    }

    fn make_prompter(&self) -> Box<dyn salt_engine::game_agent::game_agent::Prompter> {
        todo!()
    }

    fn observe_state_update(&self, game_state: salt_engine::game_state::GameStatePlayerView) {
        // for creature_slot in game_state.board().slots_iter().with_creature() {
        //     let creature = creature_slot.maybe_creature().unwrap();
        //     // self.manager.spawn_card_instance(creature);
        //     // self.sender.send("message from the agent".to_string());
        //     // self.sender.send("message from the agent".to_string());
        // }

        let message = GuiMessage::StateUpdate(game_state);
        self.sender.send(message).unwrap();
    }
}
