use crate::{
    board_slot::INPUT_EVENT_SIGNAL,
    textbox::TextBox,
    util::{self, NodeRef},
    SignalName,
};
use gdnative::{api::InputEventMouseButton, prelude::*};
use log::info;

pub(crate) const END_TURN_CLICKED_SIGNAL: SignalName = SignalName("end_turn_clicked");

#[derive(NativeClass)]
#[register_with(Self::register)]
#[inherit(Spatial)]
pub struct EndTurnButton {
    text_box: NodeRef<TextBox>,
}

impl EndTurnButton {
    fn new(_owner: &Spatial) -> Self {
        Self {
            text_box: NodeRef::<TextBox>::from_path("TextBox"),
        }
    }

    fn set_text(&self) {
        self.text_box
            .resolve_instance()
            .map_mut(|t, _| {
                t.set_text("hello");
            })
            .expect("Could not set text on EndTurnButton textbox");
    }
}

#[methods]
impl EndTurnButton {
    #[export]
    fn _ready(&mut self, owner: TRef<Spatial>) {
        info!("End turn button initialized.");

        self.text_box.init_from_parent_ref(owner);

        let mouse_collider = owner.get_node("StaticBody").unwrap();
        let mouse_collider = unsafe { mouse_collider.assume_safe_if_sane().unwrap() };
        util::connect_signal(&*mouse_collider, INPUT_EVENT_SIGNAL, owner, "input_event");
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
                owner.emit_signal(END_TURN_CLICKED_SIGNAL, &[]);
            }
        }
    }

    fn register(builder: &ClassBuilder<Self>) {
        builder.add_signal(Signal {
            name: END_TURN_CLICKED_SIGNAL.as_ref(),
            args: &[],
        });
    }
}
