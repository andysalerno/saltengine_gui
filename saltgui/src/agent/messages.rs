use salt_engine::{
    game_logic::events::ClientEventView,
    game_state::{board::BoardPos, GameStatePlayerView, PlayerId, UnitCardInstanceId},
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
        board_pos: BoardPos,
        card_instance_id: UnitCardInstanceId,
    },
    EndTurnAction,
}
