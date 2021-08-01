use crossbeam::channel::{unbounded, Receiver, Sender};
use gdnative::{
    api::{Node, PackedScene, ResourceLoader, Spatial},
    core_types::{ToVariant, Variant, Vector3},
    object::SubClass,
    prelude::{ManuallyManaged, ThreadLocal, Unique},
    Ref, TRef,
};
use log::{info, warn};
use salt_engine::{
    cards::UnitCardDefinitionView, game_agent::game_agent::GameAgent,
    game_state::UnitCardInstancePlayerView,
};
use server::{
    connection::Connection,
    messages::{FromClient, FromServer, PromptMessage},
};
use smol::net::TcpStream;

use crate::{agent::GuiAgent, hello::HelloWorld};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

const CREATURE_INSTANCE_SCENE: &str = "res://card/creature_instance.tscn";

pub(crate) fn run(sender: Sender<String>) -> Result<()> {
    smol::block_on(async {
        info!("Connecting to localhost:9000.");
        let stream = TcpStream::connect("localhost:9000").await?;
        let (connection, _) =
            async_tungstenite::client_async("ws://localhost:9000", stream).await?;

        let connection = Connection::new(connection);

        handle_connection(connection, sender).await
    })
}

async fn handle_connection(
    mut connection: Connection,
    // node: Ref<Node>,
    sender: Sender<String>,
) -> Result<()> {
    // Expect a Hello
    info!("Connected.");

    info!("Waiting for server to send my ID...");
    let my_id = match connection.recv::<FromServer>().await {
        Some(FromServer::Hello(my_id)) => my_id,
        _ => panic!("unexpected response from server"),
    };

    info!("Received my ID: {:?}", my_id);

    let mut agent: Box<dyn GameAgent> = Box::new(GuiAgent::new_with_id(sender, my_id));

    // Send Ready
    info!("Sending ready message....");
    connection.send(FromClient::Ready).await?;
    info!("Sending ready message... Done.");

    // Expect a GameStart
    info!("Waiting for GameStart message.");
    let _opponent_id = match connection.recv::<FromServer>().await {
        Some(FromServer::GameStart { opponent_id }) => opponent_id,
        other => panic!("unexpected response from server: {:?}", other),
    };
    info!("Received GameStart message.");

    // Expect the game state
    info!("Waiting for GameStateView.");
    let _gamestate_view = match connection.recv::<FromServer>().await {
        Some(FromServer::State(view)) => view,
        _ => panic!("unexpected response from server"),
    };
    info!("Received GameStateView.");

    loop {
        // Wait for signal from server that we can send an action
        info!("Waiting for next message from server.");
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
    info!("Received TurnStart message from server.");
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

pub(crate) struct NodeManager {
    root: Ref<Node>,
}

impl NodeManager {
    pub fn new(root: Ref<Node>) -> Self {
        Self { root }
    }

    pub fn spawn_card_instance(&self, card_view: &UnitCardInstancePlayerView) {
        let creature_instance = load_scene(CREATURE_INSTANCE_SCENE).unwrap();
        let creature_instance = instance_scene::<Spatial>(&creature_instance);

        creature_instance.set("title", card_view.definition().title());
        creature_instance.set("body", card_view.definition().text());

        let temp_node = unsafe { self.root.assume_safe_if_sane().expect("root node not sane") };
        temp_node.add_child(creature_instance.into_shared(), false);
    }
}

pub fn load_scene(path: &str) -> Option<Ref<PackedScene, ThreadLocal>> {
    let scene = ResourceLoader::godot_singleton().load(path, "PackedScene", false)?;

    let scene = unsafe { scene.assume_thread_local() };

    scene.cast::<PackedScene>()
}

/// Root here is needs to be the same type (or a parent type) of the node that you put in the child
///   scene as the root. For instance Spatial is used for this example.
fn instance_scene<TRoot>(scene: &PackedScene) -> Ref<TRoot, Unique>
where
    TRoot: gdnative::GodotObject<RefKind = ManuallyManaged> + SubClass<Node>,
{
    let instance = scene
        .instance(PackedScene::GEN_EDIT_STATE_DISABLED)
        .unwrap();

    let instance = unsafe { instance.assume_unique() };

    instance.try_cast::<TRoot>().unwrap()
}
