use salt_engine::game_state::GameStatePlayerView;

#[derive(Debug)]
pub(crate) enum GuiMessage {
    StateUpdate(GameStatePlayerView),
}
