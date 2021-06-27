use salt_engine::game_agent::game_agent::GameAgent;
use server::{
    connection::Connection,
    messages::{FromClient, FromServer, PromptMessage},
};
use smol::net::TcpStream;

use crate::{agent::GuiAgent, godot_log::GodotLog};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub(crate) fn run() -> Result<()> {
    smol::block_on(async {
        let stream = TcpStream::connect("localhost:9000").await?;
        let (connection, _) =
            async_tungstenite::client_async("ws://localhost:9000", stream).await?;

        let connection = Connection::new(connection);

        handle_connection(connection).await
    })
}

async fn handle_connection(mut connection: Connection) -> Result<()> {
    // Expect a Hello

    let my_id = match connection.recv::<FromServer>().await {
        Some(FromServer::Hello(my_id)) => my_id,
        _ => panic!("unexpected response from server"),
    };

    let mut agent: Box<dyn GameAgent> = Box::new(GuiAgent::new_with_id(my_id));

    // Send Ready
    connection.send(FromClient::Ready).await?;

    // Expect a GameStart
    let opponent_id = match connection.recv::<FromServer>().await {
        Some(FromServer::GameStart { opponent_id }) => opponent_id,
        other => panic!("unexpected response from server: {:?}", other),
    };

    // Expect the game state
    let gamestate_view = match connection.recv::<FromServer>().await {
        Some(FromServer::State(view)) => view,
        _ => panic!("unexpected response from server"),
    };

    loop {
        // Wait for signal from server that we can send an action
        let msg = connection
            .recv::<FromServer>()
            .await
            .expect("failed to get a response from the server");

        match msg {
            FromServer::TurnStart => handle_turn_start(&mut connection, agent.as_mut()).await?,
            FromServer::State(state) => agent.observe_state_update(state),
            _ => panic!("expected a TurnStart message, but received: {:?}", msg),
        }
    }
}

async fn handle_turn_start(connection: &mut Connection, agent: &dyn GameAgent) -> Result<()> {
    // Continuously receive actions from the client, until they end their turn.
    loop {
        // Wait for signal from server that we can send an action
        let msg = connection
            .recv::<FromServer>()
            .await
            .expect("failed to get a response from the server");

        match msg {
            FromServer::WaitingForAction(state) => {
                let player_action = agent.get_action(&state);

                let is_turn_ending = player_action.is_end_turn();

                connection
                    .send(FromClient::ClientAction(player_action))
                    .await?;

                if is_turn_ending {
                    return Ok(());
                }
            }
            FromServer::Prompt(prompt_msg, game_state) => {
                let prompter = agent.make_prompter();
                let player_input = match prompt_msg {
                    PromptMessage::PromptSlot => prompter.prompt_slot(&game_state),
                    PromptMessage::PromptCreaturePos => prompter.prompt_creature_pos(&game_state),
                    PromptMessage::PromptOpponentCreaturePos => {
                        prompter.prompt_opponent_creature_pos(&game_state)
                    }
                    PromptMessage::PromptOpponentSlot => prompter.prompt_opponent_slot(&game_state),
                    PromptMessage::PromptPlayerCreaturePos => {
                        prompter.prompt_player_creature_pos(&game_state)
                    }
                    PromptMessage::PromptPlayerSlot => prompter.prompt_player_slot(&game_state),
                };

                connection
                    .send(FromClient::PromptResponse(player_input))
                    .await?;
            }
            FromServer::State(state) => agent.observe_state_update(state),
            _ => panic!("Unexpected message from server: {:?}", msg),
        }
    }
}
