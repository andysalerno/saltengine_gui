use super::gui_message::GuiMessage;
use crate::board_slot::CLICK_RELEASED_SIGNAL;
use crate::hand::{Hand, PLAYER_HAND_CARD_ADDED_SIGNAL, PLAYER_HAND_CARD_DRAGGED};
use crate::{agent::client, hand::HandRef, util};
use cards::RicketyCannon;
use crossbeam::channel::{unbounded, Receiver, TryRecvError};
use gdnative::api::Path;
use gdnative::prelude::*;
use godot_log::GodotLog;
use log::{error, info, warn};
use salt_engine::cards::UnitCardDefinition;
use salt_engine::game_state::{MakePlayerView, PlayerId};
use salt_engine::{
    cards::UnitCardDefinitionView,
    game_state::{GameStatePlayerView, HandView, UnitCardInstancePlayerView},
};
use std::ops::Deref;
use std::thread::JoinHandle;

const CREATURE_INSTANCE_SCENE: &str = "res://card/creature_instance.tscn";
const BOARD_SLOT_PATH_PREFIX: &str = "BoardSlot";
const BOARD_PATH_RELATIVE: &str = "Board";
const PLAYER_HAND_PATH_RELATIVE: &str = "PlayerHand";

const PLAYER_HAND_NAME: &str = "PlayerHand";

#[derive(NativeClass)]
#[inherit(Node)]
pub struct World {
    recv: Receiver<GuiMessage>,
    _network_thread: JoinHandle<()>,
    state: WorldState,
}

#[derive(Debug, Default)]
struct WorldState {
    dragging_hand_card: Option<NodePath>,
}

impl World {
    fn new(_owner: &Node) -> Self {
        let (s, r) = unbounded();

        let handle = std::thread::spawn(move || {
            client::run(s).unwrap_or_else(|e| error!("Client exploded: {}", e));
        });

        info!("Websocket server started on a new thread.");

        Self {
            recv: r,
            _network_thread: handle,
            state: WorldState::default(),
        }
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

        self.add_card_to_hand(owner);
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

        info!("Connected world to signal PLAYER_HAND_CARD_ADDED_SIGNAL");
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

                info!(
                    "Connected {:?} to {} of {:?}",
                    owner.get_path(),
                    CLICK_RELEASED_SIGNAL,
                    slot_node.get_path()
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
        let message = match self.recv.try_recv() {
            Ok(msg) => msg,
            Err(TryRecvError::Disconnected) => return, // todo: display something?
            _ => return,
        };

        match message {
            GuiMessage::StateUpdate(state) => self.update_from_state(state, owner),
        }
    }

    fn board(&self, owner: TRef<Node>) -> Option<TRef<Spatial>> {
        owner
            .get_node(BOARD_PATH_RELATIVE)
            .map(|r| unsafe { r.assume_safe() })
            .map(|r| r.cast::<Spatial>().unwrap())
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
        _owner: TRef<Node>,
        dragged_card_path: Variant,
        is_ended: Variant,
    ) {
        let dragged_card_path = dragged_card_path.to_node_path();
        let is_ended = is_ended.to_bool();

        if is_ended {
            self.state.dragging_hand_card = None;
            info!("World cleared dragged card.");
        } else {
            info!("World storing new dragged card: {:?}", dragged_card_path);
            self.state.dragging_hand_card = Some(dragged_card_path);
        }
    }
}
