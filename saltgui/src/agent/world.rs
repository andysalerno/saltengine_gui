use super::bi_channel::BiChannel;
use super::messages::ToGui;
use crate::agent::bi_channel::create_channel;
use crate::agent::gui_agent::GuiAgent;
use crate::agent::messages::FromGui;
use crate::board_slot::CLICK_RELEASED_SIGNAL;
use crate::hand::{Hand, PLAYER_HAND_CARD_ADDED_SIGNAL, PLAYER_HAND_CARD_DRAGGED};
use crate::{hand::HandRef, util};
use cards::RicketyCannon;
use gdnative::api::utils::NodeExt;
use gdnative::api::{Area, Camera};
use gdnative::prelude::*;
use godot_log::GodotLog;
use log::{error, info, warn};
use salt_engine::cards::UnitCardDefinition;
use salt_engine::game_agent::game_agent::GameAgent;
use salt_engine::game_logic::ClientEventView;
use salt_engine::game_state::{MakePlayerView, PlayerId};
use salt_engine::{
    cards::UnitCardDefinitionView,
    game_state::{GameStatePlayerView, HandView, UnitCardInstancePlayerView},
};
use smol::channel::TryRecvError;
use std::thread::JoinHandle;

const CREATURE_INSTANCE_SCENE: &str = "res://card/creature_instance.tscn";
const BOARD_SLOT_PATH_PREFIX: &str = "BoardSlot";
const BOARD_PATH_RELATIVE: &str = "Board";
const PLAYER_HAND_PATH_RELATIVE: &str = "PlayerHand";
const PLAYER_HAND_NAME: &str = "PlayerHand";

#[derive(NativeClass)]
#[inherit(Node)]
pub struct World {
    _network_thread: JoinHandle<()>,
    state: WorldState,
    message_channel: BiChannel<FromGui, ToGui>,
}

#[derive(Debug, Default)]
struct WorldState {
    dragging_hand_card: Option<NodePath>,
}

impl World {
    fn new(_owner: &Node) -> Self {
        let (gui_side_channel, network_side_channel) = create_channel::<FromGui, ToGui>();

        let handle = std::thread::spawn(move || {
            smol::block_on(async {
                // The agent is a connection between the gui client and gui frontend.
                let make_agent = |player_id| {
                    Box::new(GuiAgent::new_with_id(network_side_channel, player_id))
                        as Box<dyn GameAgent>
                };

                // The client is a connection between the remote game server and the gui client.
                client::start(make_agent).await.unwrap();
            });
        });

        info!("Websocket server started on a new thread.");

        Self {
            _network_thread: handle,
            state: WorldState::default(),
            message_channel: gui_side_channel,
        }
    }

    fn observe_event(&self, event: ClientEventView, owner: TRef<Node>) {
        info!("Gui observes event: {:?}", event);
    }

    fn update_from_state(&self, state: GameStatePlayerView, owner: TRef<Node>) {
        info!("Updating from state.");
        let hand_ref = owner.get_node(PLAYER_HAND_NAME).unwrap();
        let hand_ref = unsafe { hand_ref.assume_safe() };
        let hand_ref = hand_ref.cast::<Spatial>().unwrap();
        let mut hand_ref = HandRef::new(hand_ref);
        for hand_card in state.hand().cards() {
            info!("iterating over hand_card...");
            hand_ref.add_card(hand_card);
        }
    }

    fn add_card_instance(&self, card_view: &UnitCardInstancePlayerView, owner: TRef<Node>) {
        let creature_instance = util::load_scene(CREATURE_INSTANCE_SCENE).unwrap();
        let creature_instance = util::instance_scene::<Spatial>(&creature_instance);

        creature_instance.set("title", card_view.definition().title());
        creature_instance.set("body", card_view.definition().text());

        let mut current_translation = creature_instance.translation();
        current_translation.z = -4.5;
        creature_instance.set_translation(current_translation);

        owner.add_child(creature_instance, false);
    }

    fn summon_card_on_boardslot(&self) {}
}

#[methods]
impl World {
    #[export]
    fn _ready(&self, owner: TRef<Node>) {
        GodotLog::init();
        info!("World initialized.");

        self.connect_boardslot_signals(owner);
        self.connect_hand_card_dragged(owner);
        self.connect_hand_card_added(owner);

        // self.add_card_to_hand(owner);
    }

    fn add_card_to_hand(&self, owner: TRef<Node>) {
        info!("World is adding a card to the player's hand.");
        let hand = self.player_hand(owner).unwrap();
        let mut hand = HandRef::new(hand.base());

        let dummy_card = RicketyCannon.make_instance();
        let dummy_card = dummy_card.player_view(PlayerId::new());

        hand.add_card(&dummy_card);
    }

    fn connect_hand_card_added(&self, owner: TRef<Node>) {
        let hand = self.player_hand(owner).unwrap();

        util::connect_signal(
            &*hand.base(),
            PLAYER_HAND_CARD_ADDED_SIGNAL,
            owner,
            "on_card_added_to_hand",
        );
    }

    #[export]
    fn on_card_added_to_hand(&self, owner: TRef<Node>, card_added_path: Variant) {
        let card_added_path = card_added_path.to_node_path();
        info!(
            "World observed signal card added to player hand: {:?}",
            card_added_path
        );

        let card_added = owner
            .get_node(card_added_path)
            .expect("Did not find card added at given path.");

        let card_added = unsafe { card_added.assume_safe() };

        // card_added.con
    }

    fn connect_boardslot_signals(&self, owner: TRef<Node>) {
        info!("Looking for boardslot children of {:?}", owner.get_path());

        let board = self.board(owner).unwrap();

        for slot_index in 1..19 {
            let path = format!("{}{}", BOARD_SLOT_PATH_PREFIX, slot_index);
            if let Some(slot_node) = board.get_node(&path) {
                let slot_node = unsafe { slot_node.assume_safe() };
                info!("Found board slot {:?}", slot_node.get_path());

                // Connect to all boardslots.
                util::connect_signal(
                    &*slot_node,
                    CLICK_RELEASED_SIGNAL,
                    owner,
                    "on_boardslot_click_released",
                );
            } else {
                warn!("Could not find slot node {:?}", path);
            }
        }
    }

    fn connect_hand_card_dragged(&self, owner: TRef<Node>) {
        let hand = self.player_hand(owner).unwrap();
        let hand = hand.base();

        util::connect_signal(
            &*hand,
            PLAYER_HAND_CARD_DRAGGED,
            owner,
            "on_hand_card_dragged",
        );
    }

    #[export]
    fn _process(&self, owner: TRef<Node>, _delta: f64) {
        let message = match self.message_channel.try_recv() {
            Ok(msg) => msg,
            Err(TryRecvError::Closed) => return, // todo: display something?
            _ => return,
        };

        match message {
            ToGui::StateUpdate(state) => self.update_from_state(state, owner),
            ToGui::ClientEvent(event) => self.observe_event(event, owner),
        }
    }

    fn board(&self, owner: TRef<Node>) -> Option<TRef<Spatial>> {
        unsafe { owner.as_ref().get_node_as::<Spatial>(BOARD_PATH_RELATIVE) }
    }

    fn camera(&self, owner: TRef<Node>) -> Option<TRef<Camera>> {
        unsafe { owner.as_ref().get_node_as::<Camera>("Camera") }
    }

    fn player_hand(&self, owner: TRef<Node>) -> Option<RefInstance<Hand, Shared>> {
        owner
            .get_node(PLAYER_HAND_PATH_RELATIVE)
            .map(|r| unsafe { r.assume_safe() })
            .map(|r| r.cast::<Spatial>().unwrap())
            .map(|r| r.cast_instance::<Hand>().unwrap())
    }

    #[export]
    fn my_method(&self, _owner: TRef<Node>) {
        info!("Invoked my_method.");
    }

    /// Invoked by a signal whenever a boardslot has a "click release" action.
    /// If there's currently a "dragged card" active, this means the player
    /// is attempting to summon the dragged card to the given boardslot.
    #[export]
    fn on_boardslot_click_released(&self, owner: TRef<Node>, data: Variant) {
        info!(
            "world on_boardslot_click_released for {:?} with data: {:?}",
            owner.get_path(),
            data
        );
    }

    /// Invoked by a signal whenever a card in the player's hand begins or ends dragging.
    #[export]
    fn on_hand_card_dragged(
        &mut self,
        owner: TRef<Node>,
        dragged_card_path: Variant,
        is_ended: Variant,
        mouse_pos_2d: Variant,
    ) {
        let dragged_card_path = dragged_card_path.to_node_path();
        let is_ended = is_ended.to_bool();

        if is_ended {
            self.state.dragging_hand_card = None;
            info!("World cleared dragged card.");
            let mouse_pos = mouse_pos_2d.to_vector2();
            if let Some(path) = self.find_overlapping_boardslot(owner, mouse_pos) {
                self.message_channel
                    .send_blocking(FromGui::SummonFromHandToSlotRequest(path.to_string()))
                    .expect("Failed to send request from guid to network thread.");
                info!("User released card over boardslot {:?}", path);
            } else {
                info!("User released card, but not over a boardslot.");
            }
        } else {
            info!("World storing new dragged card: {:?}", dragged_card_path);
            self.state.dragging_hand_card = Some(dragged_card_path);
        }
    }

    fn find_overlapping_boardslot(
        &self,
        owner: TRef<Node>,
        mouse_pos: Vector2,
    ) -> Option<NodePath> {
        // Cast ray from the moust position to the BoardSlot layer.
        let camera = self.camera(owner).unwrap();

        let project_from = camera.project_ray_origin(mouse_pos);
        let project_to = project_from + (camera.project_ray_normal(mouse_pos) * 10.);

        let world = camera.get_world().unwrap();
        let world = unsafe { world.assume_safe() };
        let space_state = world.direct_space_state().unwrap();
        let space_state = unsafe { space_state.assume_safe() };

        let exclude = VariantArray::new_shared();
        let collision_mask: i64 = 2;
        let collision = space_state.intersect_ray(
            project_from,
            project_to,
            exclude,
            collision_mask,
            false,
            true,
        );

        collision
            .get("collider")
            .try_to_object::<Area>()
            .map(|area| {
                let area = unsafe { area.assume_safe() };
                let parent = area.get_parent().unwrap();
                let parent = unsafe { parent.assume_safe() };
                parent.get_path()
            })
    }
}
