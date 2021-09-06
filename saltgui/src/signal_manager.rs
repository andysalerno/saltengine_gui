use crate::{board_slot::INPUT_EVENT_SIGNAL, util, SignalName};
use gdnative::{api::InputEventMouseButton, prelude::*};
use log::info;

#[derive(NativeClass)]
#[register_with(Self::register)]
#[inherit(Object)]
pub(crate) struct SignalManager;

impl SignalManager {
    fn new(_owner: &Object) -> Self {
        Self {}
    }
}

#[methods]
impl SignalManager {
    #[export]
    fn _ready(&self, _owner: TRef<Object>) {
        info!("Signal manager generated.");
    }
}
