use crate::{
    board_slot::{BoardSlot, SlotPos},
    util::NodeRef,
};
use gdnative::prelude::*;
use log::info;
use salt_engine::game_state::{
    board::{BoardPos, RowId},
    UnitCardInstancePlayerView,
};

const BOARD_SLOT_COUNT: usize = 24;
const BOARD_SLOT_PATH_PREFIX: &str = "BoardSlot";

#[derive(NativeClass, Debug, Default)]
#[inherit(Spatial)]
pub struct Board {
    slots: Vec<NodeRef<BoardSlot, Spatial>>,
}

impl Board {
    fn new(owner: TRef<Spatial>) -> Self {
        // let slot = NodeRef::<BoardSlot, Spatial>::from_parent_ref("Test", owner.upcast::<Node>());
        Self { slots: vec![] }
    }

    pub(crate) fn get_slot(&self, pos: SlotPos) -> RefInstance<BoardSlot, Shared> {
        self.slots
            .iter()
            .filter(|s| s.resolve_instance().map(|slot, _| slot.pos()).unwrap() == pos)
            .map(|s| s.resolve_instance())
            .next()
            .unwrap()

        // for slot_index in 1..=BOARD_SLOT_COUNT {
        //     let path = format!("{}{}", BOARD_SLOT_PATH_PREFIX, slot_index);
        //     if let Some(slot_node) = board.base().get_node(&path) {
        //     } else {
        //     }
        // }
    }

    pub fn add_card(&mut self, card: &UnitCardInstancePlayerView, owner: TRef<Spatial>) {
        // info!(
        //     "Hand is receiving a card: {} {:?}",
        //     card.definition().title(),
        //     card.id()
        // );

        // let card_instance = CardInstance::new_instance();

        // // let offset = hand.map(|n, _| n.hand_len).unwrap() as f32 * OFFSET_DIST_MULTIPLIER;
        // let offset = self.hand_len as f32 * OFFSET_DIST_MULTIPLIER;

        // card_instance
        //     .map_mut(|c, n| {
        //         let def = card.definition();
        //         c.set_title(def.title());
        //         c.set_body(def.text());

        //         c.set_view(card.clone());

        //         n.translate(Vector3::new(offset, 0., 0.));

        //         util::connect_signal(n, CARD_DRAGGED, owner, "on_card_dragged");
        //     })
        //     .unwrap();

        // self.hand_len += 1;

        // let card_instance = card_instance.into_base();
        // let card_instance = card_instance.into_shared();
        // owner.add_child(card_instance, false);

        // let card_instance = unsafe { card_instance.assume_safe() };
        // let card_path = card_instance.get_path();

        // owner.emit_signal(PLAYER_HAND_CARD_ADDED_SIGNAL, &[card_path.to_variant()]);

        // info!("Added card {:?} to PlayerHand.", card_path);
    }

    fn boardslot_from_pos(&self, pos: SlotPos) -> usize {
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

    fn init_board_slot_pos(&mut self, owner: TRef<Spatial>) {
        info!("Initializing board slots...");
        let row_len = BOARD_SLOT_COUNT / 4;

        let owner = owner.upcast::<Node>();

        // Opponent back
        for i in 0..row_len {
            let pos = SlotPos {
                index: i,
                row_id: RowId::BackRow,
                is_friendly: false,
            };
            let slot_index = self.boardslot_from_pos(pos);
            let slot_path = format!("{}{}", BOARD_SLOT_PATH_PREFIX, slot_index);
            let slot: NodeRef<BoardSlot, Spatial> = NodeRef::from_parent_ref(&slot_path, owner);
            let slot_instance = slot.resolve_instance();
            slot_instance
                .map_mut(|a, _| {
                    a.set_pos(pos);
                })
                .unwrap();

            self.slots.push(slot);
        }

        // Opponent front
        for i in 0..row_len {
            let pos = SlotPos {
                index: i,
                row_id: RowId::FrontRow,
                is_friendly: false,
            };
            let slot_index = self.boardslot_from_pos(pos);
            let slot_path = format!("{}{}", BOARD_SLOT_PATH_PREFIX, slot_index);
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
            let slot_index = self.boardslot_from_pos(pos);
            let slot_path = format!("{}{}", BOARD_SLOT_PATH_PREFIX, slot_index);
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
            let slot_index = self.boardslot_from_pos(pos);
            let slot_path = format!("{}{}", BOARD_SLOT_PATH_PREFIX, slot_index);
            let slot: NodeRef<BoardSlot, Spatial> = NodeRef::from_parent_ref(&slot_path, owner);
            let slot = slot.resolve_instance();
            slot.map_mut(|a, _| {
                a.set_pos(pos);
            })
            .unwrap();
        }

        info!("Done initializing board slots.");
    }
}

#[methods]
impl Board {
    #[export]
    fn _ready(&mut self, owner: TRef<Spatial>) {
        // for slot_index in 1..=BOARD_SLOT_COUNT {
        //     let path = format!("{}{}", BOARD_SLOT_PATH_PREFIX, slot_index);
        //     if let Some(slot_node) = board.base().get_node(&path) {
        //     } else {
        //     }
        // }

        self.init_board_slot_pos(owner);
    }
}
