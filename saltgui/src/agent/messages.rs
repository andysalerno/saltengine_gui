use salt_engine::{
    game_logic::ClientEventView,
    game_state::{GameStatePlayerView, UnitCardInstanceId},
};

#[derive(Debug, Clone)]
pub(crate) enum ToGui {
    StateUpdate(GameStatePlayerView),
    ClientEvent(ClientEventView),
}

#[derive(Debug, Clone)]
pub(crate) enum FromGui {
    SummonFromHandToSlotRequest {
        slot_path: String,
        card_instance_id: UnitCardInstanceId,
    },
    EndTurnAction,
}
