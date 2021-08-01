use gdnative::{api::Node, Ref};
use salt_engine::{
    game_agent::game_agent::GameAgent,
    game_logic::{ClientGameEvent, EndTurnEvent},
    game_state::{board::BoardView, IterAddons, PlayerId},
};

use crate::client::NodeManager;

pub(crate) struct GuiAgent {
    player_id: PlayerId,
    manager: NodeManager,
}

impl GuiAgent {
    pub fn new_with_id(manager: NodeManager, player_id: PlayerId) -> Self {
        Self { manager, player_id }
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

    fn observe_state_update(&self, _game_state: salt_engine::game_state::GameStatePlayerView) {
        for creature_slot in _game_state.board().slots_iter().with_creature() {
            let creature = creature_slot.maybe_creature().unwrap();
            self.manager.spawn_card_instance(creature);
        }
    }
}
