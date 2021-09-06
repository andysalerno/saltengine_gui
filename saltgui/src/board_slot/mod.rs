use crate::{
    textbox::TextBox,
    util::{self, NodeRef},
    SignalName,
};
use gdnative::{api::InputEventMouseButton, prelude::*};
use log::info;
use salt_engine::{
    game_logic::events::SummonCreatureFromHandClientEvent, game_state::UnitCardInstancePlayerView,
};

#[derive(NativeClass)]
#[register_with(Self::register)]
#[inherit(Spatial)]
pub struct BoardSlot {
    textbox: Option<NodeRef<TextBox>>,
}

/// Emitted when a click is released over this `BoardSlot`.
pub(crate) const CLICK_RELEASED_SIGNAL: SignalName = SignalName("click_released");

/// Internal signal from Godot emitted when there is an input event.
pub(crate) const INPUT_EVENT_SIGNAL: SignalName = SignalName("input_event");

impl BoardSlot {
    fn new(_owner: &Spatial) -> Self {
        Self { textbox: None }
    }
}

impl BoardSlot {
    // pub fn receive_summon(&self, card_view: UnitCardInstancePlayerView) {
    pub fn receive_summon(&self, card_view: SummonCreatureFromHandClientEvent) {
        let textbox = self.textbox.as_ref().unwrap().resolve_instance();
        textbox
            .map_mut(|b, a| {
                b.set_text("I got a summon!!");
            })
            .unwrap();
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

        {
            let text_box = owner.get_node("TextBox").unwrap();
            let r: NodeRef<TextBox> = NodeRef::from_existing("TextBox", text_box);
            self.textbox = Some(r);
        }

        // self.textbox.and_then(|t| {
        //     let i = t.resolve_instance();
        // });
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
