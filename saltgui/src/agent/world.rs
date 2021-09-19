use super::bi_channel::BiChannel;
use super::messages::ToGui;
use crate::agent::bi_channel::create_channel;
use crate::agent::gui_agent::GuiClient;
use crate::agent::messages::FromGui;
use crate::board_slot::{BoardSlot, SlotPos, CLICK_RELEASED_SIGNAL};
use crate::card_instance::CardInstance;
use crate::end_turn_button::{EndTurnButton, END_TURN_CLICKED_SIGNAL};
use crate::gui_mana_counter::ManaCounter;
use crate::hand::{Hand, PLAYER_HAND_CARD_DRAGGED};
use crate::util;
use crate::util::NodeRef;
use gdnative::api::utils::NodeExt;
use gdnative::api::{Area, Camera};
use gdnative::prelude::*;
use godot_log::GodotLog;
use log::{info, warn};
use salt_engine::game_logic::events::{
    AddCardToHandClientEvent, ClientEventView, CreatureSetClientEvent,
};
use salt_engine::game_runner::GameClient;
use salt_engine::game_state::board::RowId;
use salt_engine::game_state::GameStatePlayerView;
use salt_engine::game_state::PlayerId;
use smol::channel::TryRecvError;
use std::thread::JoinHandle;

const BOARD_SLOT_COUNT: usize = 24;
const BOARD_SLOT_PATH_PREFIX: &str = "BoardSlot";
const BOARD_PATH_RELATIVE: &str = "Board";
const PLAYER_HAND_PATH_RELATIVE: &str = "PlayerHand";
const END_TURN_BUTTON: &str = "EndTurnButton";
const MANA_DISPLAY: &str = "ManaCounter";

/// State for maintaining certain UI-specific values over the course of the game.
#[derive(Debug, Default)]
struct WorldState {
    player_id: Option<PlayerId>,
    opponent_id: Option<PlayerId>,
    dragging_hand_card: Option<NodePath>,
    card_to_summon: Option<(NodeRef<BoardSlot, Spatial>, NodePath)>,
    player_mana_limit: usize,
    enemy_mana_limit: usize,
    player_mana_count: usize,
    enemy_mana_count: usize,
}

/// The parent world logic, with ownership over every aspect of the UI.
#[derive(NativeClass)]
#[inherit(Node)]
pub struct World {
    _network_thread: JoinHandle<()>,
    state: WorldState,
    message_channel: BiChannel<FromGui, ToGui>,
    end_turn_button: NodeRef<EndTurnButton, Spatial>,
    mana_display: NodeRef<ManaCounter, Control>,
    player_hand: NodeRef<Hand, Spatial>,
}

impl World {
    fn new(_owner: &Node) -> Self {
        let (gui_side_channel, network_side_channel) = create_channel::<FromGui, ToGui>();

        let handle = std::thread::spawn(move || {
            smol::block_on(async {
                // The agent is a connection between the gui client and gui frontend.
                let make_agent = |player_id| {
                    Box::new(GuiClient::new_with_id(network_side_channel, player_id))
                        as Box<dyn GameClient>
                };

                // The client is a connection between the remote game server and the gui client.
                websocket_client::start(make_agent).await.unwrap();
            });
        });

        info!("Websocket server started on a new thread.");

        Self {
            _network_thread: handle,
            state: WorldState::default(),
            message_channel: gui_side_channel,
            end_turn_button: NodeRef::<EndTurnButton, Spatial>::from_path(END_TURN_BUTTON),
            mana_display: NodeRef::<ManaCounter, Control>::from_path(MANA_DISPLAY),
            player_hand: NodeRef::<Hand, Spatial>::from_path(PLAYER_HAND_PATH_RELATIVE),
        }
    }

    /// Invoked each frame where there is a message from the server with a notification update.
    fn observe_notifier_event(&mut self, event: ClientEventView, owner: TRef<Node>) {
        info!("Gui observes event: {:?}", event);

        match event {
            ClientEventView::AddCardToHand(e) => self.observe_add_card_to_hand(e, owner),
            ClientEventView::UnitSet(e) => self.observe_creature_set_event(e, owner),
            ClientEventView::SummonCreatureFromHand(_) => {}
            ClientEventView::TurnEnded(id) => self.observe_turn_ended(id, owner),
            ClientEventView::TurnStarted(id) => self.observe_turn_started(id, owner),
            ClientEventView::PlayerGainMana(player, count) => {
                self.observe_player_gain_mana(player, count, owner);
            }
            ClientEventView::PlayerSpendMana {
                player_id,
                spent_mana_count,
            } => self.observe_player_spend_mana(player_id, spent_mana_count, owner),
        }
    }

    fn observe_turn_ended(&self, _player: PlayerId, _owner: TRef<Node>) {
        // nothing currently
    }

    fn observe_player_spend_mana(
        &mut self,
        player_id: PlayerId,
        spent_mana_count: usize,
        _owner: TRef<Node>,
    ) {
        if player_id == self.state.player_id.unwrap() {
            self.state.player_mana_count -= spent_mana_count;
            let unused = self.state.player_mana_count;
            let limit = self.state.player_mana_limit;
            self.mana_display
                .resolve_instance()
                .map(|c, _| {
                    c.set_display(unused, limit);
                })
                .expect("Unable to set mana label");
        } else {
            self.state.enemy_mana_count -= spent_mana_count;
            let limit = self.state.enemy_mana_limit;
            let unused = self.state.enemy_mana_count;
            self.mana_display
                .resolve_instance()
                .map(|c, _| {
                    c.set_display(unused, limit);
                })
                .expect("Unable to set mana label");
        }
    }

    fn observe_player_gain_mana(
        &mut self,
        player: PlayerId,
        mana_gain_count: usize,
        _owner: TRef<Node>,
    ) {
        if player == self.state.player_id.unwrap() {
            self.state.player_mana_limit += mana_gain_count;
            let unused = self.state.player_mana_count;
            let limit = self.state.player_mana_limit;
            self.mana_display
                .resolve_instance()
                .map(|c, _| {
                    c.set_display(unused, limit);
                })
                .expect("Unable to set mana label");
        } else {
            self.state.enemy_mana_limit += mana_gain_count;
            let limit = self.state.enemy_mana_limit;
            let unused = self.state.enemy_mana_count;
            self.mana_display
                .resolve_instance()
                .map(|c, _| {
                    c.set_display(unused, limit);
                })
                .expect("Unable to set mana label");
        }
    }

    fn observe_turn_started(&mut self, player: PlayerId, _owner: TRef<Node>) {
        let button_text;
        if player == self.state.player_id.unwrap() {
            // TODO: this is a big hack - we add +1 because we know we gain +1 mana upon turn start.
            // But in reality, we should be responding to the "gain mana" event, not the "turn start" event.
            self.state.player_mana_count = self.state.player_mana_limit + 1;
            button_text = "End turn";
        } else {
            self.state.enemy_mana_count = self.state.enemy_mana_limit + 1;
            button_text = "(Enemy turn)";
        };

        let button = self.end_turn_button.resolve_instance();
        button
            .map(|t, _| t.set_text(button_text))
            .expect("Could not set_text on textbox");
    }

    fn observe_add_card_to_hand(&self, event: AddCardToHandClientEvent, _owner: TRef<Node>) {
        info!("World is adding a card to the player's hand.");
        let hand = self.player_hand.resolve_instance();

        hand.map_mut(|h, n| {
            h.add_card(&event.card, n);
        })
        .expect("failed to add card to hand");
    }

    fn observe_creature_set_event(&self, event: CreatureSetClientEvent, owner: TRef<Node>) {
        info!("World saw a summon event.");
        let slot_pos = SlotPos {
            row_id: event.pos.row_id,
            is_friendly: event.pos.player_id == self.state.player_id.unwrap(),
            index: event.pos.row_index,
        };
        let slot_index = self.boardslot_from_pos(owner, slot_pos);
        let slot_path = format!(
            "{}/{}{}",
            BOARD_PATH_RELATIVE, BOARD_SLOT_PATH_PREFIX, slot_index
        );
        let slot: NodeRef<BoardSlot, Spatial> = NodeRef::from_parent_ref(&slot_path, owner);

        let slot = slot.resolve_instance();
        slot.map_mut(|a, _| a.receive_summon(event))
            .expect("Failed to receive summon for slot");
    }

    /// Invoked each frame where there is a message from the server with a state update.
    fn update_from_state(&mut self, state: GameStatePlayerView, _owner: TRef<Node>) {
        if self.state.opponent_id.is_none() {
            self.state.opponent_id = Some(state.opponent_id());
            info!("My opponent is: {:?}", state.opponent_id());
        }
    }

    /// Get a card instance given its path.
    fn card_instance<'a>(
        &self,
        path: impl AsRef<str>,
        owner: TRef<'a, Node>,
    ) -> Option<RefInstance<'a, CardInstance, Shared>> {
        util::get_as(path, owner)
    }

    fn board(&self, owner: TRef<Node>) -> Option<TRef<Spatial>> {
        unsafe { owner.as_ref().get_node_as::<Spatial>(BOARD_PATH_RELATIVE) }
    }

    fn camera(&self, owner: TRef<Node>) -> Option<TRef<Camera>> {
        unsafe { owner.as_ref().get_node_as::<Camera>("Camera") }
    }

    /// Given a `SlotPos`, returns its corresponding board slot number.
    fn boardslot_from_pos(&self, _owner: TRef<Node>, pos: SlotPos) -> usize {
        let row_len = BOARD_SLOT_COUNT / 4;

        let offset = if pos.is_friendly {
            let player_offset = row_len * 2;

            let row_offset = match pos.row_id {
                salt_engine::game_state::board::RowId::FrontRow => 0,
                salt_engine::game_state::board::RowId::BackRow => row_len,
                salt_engine::game_state::board::RowId::Hero => todo!("hero not done yet"),
            };

            let index_offset = pos.index;

            player_offset + row_offset + index_offset
        } else {
            let player_offset = 0;

            let row_offset = match pos.row_id {
                salt_engine::game_state::board::RowId::FrontRow => row_len,
                salt_engine::game_state::board::RowId::BackRow => 0,
                salt_engine::game_state::board::RowId::Hero => todo!("hero not done yet"),
            };

            let index_offset = pos.index;

            player_offset + row_offset + index_offset
        };

        // In the Godot world, slots begin at index 1 instead of 0
        offset + 1
    }
}

#[methods]
impl World {
    /// Invoked by Godot when this instance is done initializing.
    #[export]
    fn _ready(&mut self, owner: TRef<Node>) {
        GodotLog::init();
        info!("World initialized.  Hello.");

        self.end_turn_button.init_from_parent_ref(owner);
        self.mana_display.init_from_parent_ref(owner);
        self.player_hand.init_from_parent_ref(owner);

        self.connect_boardslot_signals(owner);
        self.connect_hand_card_dragged(owner);
        self.connect_end_turn_clicked(owner);
        self.init_board_slot_pos(owner);
    }

    /// Invoked every frame by Godot.
    #[export]
    fn _process(&mut self, owner: TRef<Node>, _delta: f64) {
        if let Some((slot_path, card_path)) = self.state.card_to_summon.take() {
            info!("Summoning card from within _process().");
            let card_inst = self
                .card_instance(card_path.to_string(), owner)
                .expect("Could not find card instance.");

            info!("using map to do the thing........");
            let card_instance_id = card_inst.map(|a, _| a.expect_view().id()).unwrap();

            let slot_pos = slot_path.resolve_instance().map(|a, b| a.pos()).unwrap();
            let board_pos = slot_pos.into_board_slot(self.state.player_id.unwrap());

            self.message_channel
                .send_blocking(FromGui::SummonFromHandToSlotRequest {
                    board_pos,
                    card_instance_id,
                })
                .expect("Failed to send request from guid to network thread.");

            let hand_card: NodeRef<CardInstance, Spatial> =
                NodeRef::from_parent_ref(card_path.to_string(), owner);

            let hand_card = hand_card.resolve_instance();
            hand_card.base().queue_free();
        }

        let message = match self.message_channel.try_recv() {
            Ok(msg) => msg,
            Err(TryRecvError::Closed) => return, // todo: display something?
            _ => return,
        };

        match message {
            ToGui::StateUpdate(state) => self.update_from_state(state, owner),
            ToGui::ClientEvent(event) => self.observe_notifier_event(event, owner),
            ToGui::PlayerIdSet(player_id) => self.state.player_id = Some(player_id),
        }
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
            if let Some(slot_path) = self.find_overlapping_boardslot(owner, mouse_pos) {
                self.state.card_to_summon = Some((slot_path, dragged_card_path));
            } else {
                info!("User released card, but not over a boardslot.");
            }
        } else {
            info!("World storing new dragged card: {:?}", dragged_card_path);
            self.state.dragging_hand_card = Some(dragged_card_path);
        }
    }

    #[export]
    fn on_end_turn_clicked(&self, _owner: TRef<Node>) {
        info!("The world sees taht end turn was clicked.");
        self.message_channel
            .send_blocking(FromGui::EndTurnAction)
            .unwrap();
    }

    fn init_board_slot_pos(&self, owner: TRef<Node>) {
        let row_len = BOARD_SLOT_COUNT / 4;

        // Opponent back
        for i in 0..row_len {
            let pos = SlotPos {
                index: i,
                row_id: RowId::BackRow,
                is_friendly: false,
            };
            let slot_index = self.boardslot_from_pos(owner, pos);
            let slot_path = format!(
                "{}/{}{}",
                BOARD_PATH_RELATIVE, BOARD_SLOT_PATH_PREFIX, slot_index
            );
            let slot: NodeRef<BoardSlot, Spatial> = NodeRef::from_parent_ref(&slot_path, owner);
            let slot = slot.resolve_instance();
            slot.map_mut(|a, _| {
                a.set_pos(pos);
            })
            .unwrap();
        }

        // Opponent front
        for i in 0..row_len {
            let pos = SlotPos {
                index: i,
                row_id: RowId::FrontRow,
                is_friendly: false,
            };
            let slot_index = self.boardslot_from_pos(owner, pos);
            let slot_path = format!(
                "{}/{}{}",
                BOARD_PATH_RELATIVE, BOARD_SLOT_PATH_PREFIX, slot_index
            );
            let slot: NodeRef<BoardSlot, Spatial> = NodeRef::from_parent_ref(&slot_path, owner);
            let slot = slot.resolve_instance();
            slot.map_mut(|a, _| {
                a.set_pos(pos);
            })
            .unwrap();
        }

        // Player front
        for i in 0..row_len {
            let pos = SlotPos {
                index: i,
                row_id: RowId::FrontRow,
                is_friendly: true,
            };
            let slot_index = self.boardslot_from_pos(owner, pos);
            let slot_path = format!(
                "{}/{}{}",
                BOARD_PATH_RELATIVE, BOARD_SLOT_PATH_PREFIX, slot_index
            );
            let slot: NodeRef<BoardSlot, Spatial> = NodeRef::from_parent_ref(&slot_path, owner);
            let slot = slot.resolve_instance();
            slot.map_mut(|a, _| {
                a.set_pos(pos);
            })
            .unwrap();
        }

        // Player back
        for i in 0..row_len {
            let pos = SlotPos {
                index: i,
                row_id: RowId::BackRow,
                is_friendly: true,
            };
            let slot_index = self.boardslot_from_pos(owner, pos);
            let slot_path = format!(
                "{}/{}{}",
                BOARD_PATH_RELATIVE, BOARD_SLOT_PATH_PREFIX, slot_index
            );
            let slot: NodeRef<BoardSlot, Spatial> = NodeRef::from_parent_ref(&slot_path, owner);
            let slot = slot.resolve_instance();
            slot.map_mut(|a, _| {
                a.set_pos(pos);
            })
            .unwrap();
        }
    }

    fn connect_end_turn_clicked(&self, owner: TRef<Node>) {
        // let hand = self.player_hand(owner).unwrap();
        let button: RefInstance<EndTurnButton, Shared> =
            util::get_as(END_TURN_BUTTON, owner).expect("Could not find end turn button node.");

        util::connect_signal(
            &*button.base(),
            END_TURN_CLICKED_SIGNAL,
            owner,
            "on_end_turn_clicked",
        );
    }

    fn connect_boardslot_signals(&self, owner: TRef<Node>) {
        info!("Looking for boardslot children of {:?}", owner.get_path());

        let board = self.board(owner).unwrap();

        for slot_index in 1..=BOARD_SLOT_COUNT {
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
        // let hand = self.player_hand(owner).unwrap();
        let hand = self.player_hand.resolve_instance();
        let hand = hand.base();

        util::connect_signal(
            &*hand,
            PLAYER_HAND_CARD_DRAGGED,
            owner,
            "on_hand_card_dragged",
        );
    }

    fn find_overlapping_boardslot(
        &self,
        owner: TRef<Node>,
        mouse_pos: Vector2,
    ) -> Option<NodeRef<BoardSlot, Spatial>> {
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
                let parent_path = parent.get_path();

                info!("NodeRef creating from path: {:?}", parent_path);

                let node_ref: NodeRef<BoardSlot, Spatial> =
                    NodeRef::from_parent_ref(parent_path.to_string(), owner);
                node_ref
            })
    }
}
