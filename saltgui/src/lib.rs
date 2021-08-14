mod agent;
mod board_slot;
mod card_instance;
mod hand;
mod util;

use agent::hello::HelloWorld;
use board_slot::BoardSlot;
use card_instance::CardInstance;
use gdnative::prelude::*;
use godot_log::GodotLog;
use hand::Hand;

fn init(handle: InitHandle) {
    GodotLog::init();
    handle.add_class::<CardInstance>();
    handle.add_class::<HelloWorld>();
    handle.add_class::<Hand>();
    handle.add_class::<BoardSlot>();
}

// Macro that creates the entry-points of the dynamic library.
godot_init!(init);
