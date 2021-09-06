#![deny(clippy::all, nonstandard_style, future_incompatible)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::needless_pass_by_value,
    clippy::unused_self,
    clippy::cast_lossless
)]
mod agent;
mod board_slot;
mod card_instance;
mod end_turn_button;
mod hand;
mod signal_manager;
mod textbox;
mod util;

use agent::world::World;
use board_slot::BoardSlot;
use card_instance::CardInstance;
use end_turn_button::EndTurnButton;
use gdnative::prelude::*;
use godot_log::GodotLog;
use hand::Hand;
use signal_manager::SignalManager;
use textbox::TextBox;

fn init(handle: InitHandle) {
    GodotLog::init();
    handle.add_class::<CardInstance>();
    handle.add_class::<World>();
    handle.add_class::<Hand>();
    handle.add_class::<BoardSlot>();
    handle.add_class::<EndTurnButton>();
    handle.add_class::<SignalManager>();
    handle.add_class::<TextBox>();
}

// Macro that creates the entry-points of the dynamic library.
godot_init!(init);

/// The typed name of a signal.
#[derive(Copy, Clone, Debug)]
struct SignalName(&'static str);

impl std::fmt::Display for SignalName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl SignalName {
    pub fn as_ref(&self) -> &str {
        self.0
    }
}

impl From<SignalName> for GodotString {
    fn from(signal_name: SignalName) -> Self {
        signal_name.0.into()
    }
}
