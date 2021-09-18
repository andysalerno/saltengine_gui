use crate::{
    textbox::TextBox,
    util::{self, NodeRef},
    SignalName,
};
use gdnative::{api::InputEventMouseButton, prelude::*};
use log::info;
use salt_engine::{
    cards::UnitCardDefinitionView,
    game_logic::events::CreatureSetClientEvent,
    game_state::{
        board::{BoardPos, RowId},
        PlayerId,
    },
};

#[derive(NativeClass, Debug)]
#[register_with(Self::register)]
#[inherit(Spatial)]
pub struct BoardSlot {
    textbox: NodeRef<TextBox, Spatial>,
    board_pos: Option<SlotPos>,
}

/// Emitted when a click is released over this `BoardSlot`.
pub(crate) const CLICK_RELEASED_SIGNAL: SignalName = SignalName("click_released");

/// Internal signal from Godot emitted when there is an input event.
pub(crate) const INPUT_EVENT_SIGNAL: SignalName = SignalName("input_event");

/// Just like `BoardSlot`, except agnostic to the player's ID.
#[derive(Debug, Copy, Clone)]
pub struct SlotPos {
    pub is_friendly: bool,
    pub row_id: RowId,
    pub index: usize,
}

impl SlotPos {
    pub fn into_board_slot(self, player_id: PlayerId) -> BoardPos {
        BoardPos::new(player_id, self.row_id, self.index)
    }
}

impl BoardSlot {
    fn new(_owner: &Spatial) -> Self {
        Self {
            textbox: NodeRef::from_path("TextBox"),
            board_pos: None,
        }
    }

    pub fn receive_summon(&self, card_view: CreatureSetClientEvent) {
        let title = card_view.card.definition().title();
        let attack = card_view.card.attack();
        let health = card_view.card.health();

        let text = format!("{}\n{}/{}", title, attack, health);

        let textbox = self.textbox.resolve_instance();
        textbox
            .map_mut(|i, _| {
                i.set_text(&text);
            })
            .unwrap();
    }

    pub fn set_pos(&mut self, pos: SlotPos) {
        info!("Looks like my pos is: {:?}", pos);
        self.board_pos = Some(pos);
    }

    pub fn pos(&self) -> SlotPos {
        self.board_pos.unwrap()
    }
}

#[methods]
impl BoardSlot {
    #[export]
    fn _ready(&mut self, owner: TRef<Spatial>) {
        {
            let mouse_collider = owner.get_node("Area").unwrap();
            let mouse_collider = unsafe { mouse_collider.assume_safe_if_sane().unwrap() };
            util::connect_signal(&*mouse_collider, INPUT_EVENT_SIGNAL, owner, "input_event");
        }

        self.textbox.init_from_parent_ref(owner);
    }

    #[export]
    fn input_event(
        &mut self,
        owner: TRef<Spatial>,
        _camera: Variant,
        mouse_event: Variant,

        // Clicked position in world-space.
        _click_pos: Variant,
        _click_normal: Variant,
        _shape_idx: Variant,
    ) {
        if let Some(event) = mouse_event.try_to_object::<InputEventMouseButton>() {
            let click = unsafe { event.assume_safe() };
            if !click.is_pressed() {
                owner.emit_signal(CLICK_RELEASED_SIGNAL, &[owner.get_path().to_variant()]);
            }
        }
    }

    fn register(builder: &ClassBuilder<Self>) {
        builder.add_signal(Signal {
            name: CLICK_RELEASED_SIGNAL.as_ref(),
            args: &[SignalArgument {
                name: "path",
                default: Variant::from_str("<empty_default>"),
                export_info: ExportInfo::new(VariantType::GodotString),
                usage: PropertyUsage::DEFAULT,
            }],
        });
    }
}
