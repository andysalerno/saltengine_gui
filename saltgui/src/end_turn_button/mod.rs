use crate::{board_slot::INPUT_EVENT_SIGNAL, util, SignalName};
use gdnative::{api::InputEventMouseButton, prelude::*};
use log::info;

pub(crate) const END_TURN_CLICKED_SIGNAL: SignalName = SignalName("end_turn_clicked");

#[derive(NativeClass)]
#[register_with(Self::register)]
#[inherit(Spatial)]
pub struct EndTurnButton {}

impl EndTurnButton {
    fn new(_owner: &Spatial) -> Self {
        Self {}
    }
}

#[methods]
impl EndTurnButton {
    #[export]
    fn _ready(&self, owner: TRef<Spatial>) {
        info!("End turn button initialized.");

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
