use salt_engine::{game_logic::ClientEventView, game_state::GameStatePlayerView};

#[derive(Debug, Clone)]
pub(crate) enum ToGui {
    StateUpdate(GameStatePlayerView),
    ClientEvent(ClientEventView),
}

#[derive(Debug, Clone)]
pub(crate) enum FromGui {
    SummonFromHandToSlotRequest(String),
}
