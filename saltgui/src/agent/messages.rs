use salt_engine::{
    game_logic::events::ClientEventView,
    game_state::{GameStatePlayerView, PlayerId, UnitCardInstanceId},
};

#[derive(Debug, Clone)]
pub(crate) enum ToGui {
    StateUpdate(GameStatePlayerView),
    ClientEvent(ClientEventView),
    PlayerIdSet(PlayerId),
}

#[derive(Debug, Clone)]
pub(crate) enum FromGui {
    SummonFromHandToSlotRequest {
        slot_path: String,
        card_instance_id: UnitCardInstanceId,
    },
    EndTurnAction,
}
