use gdnative::prelude::*;
use log::info;

use crate::{board_slot::INPUT_EVENT_SIGNAL, util};

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
        info!("input_event()");
    }
}
