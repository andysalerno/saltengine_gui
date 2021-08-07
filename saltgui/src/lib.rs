mod agent;
mod card_instance;
mod hand;

use agent::hello::HelloWorld;
use card_instance::CardInstance;
use gdnative::prelude::*;
use godot_log::GodotLog;
use hand::Hand;

fn init(handle: InitHandle) {
    GodotLog::init();
    handle.add_class::<CardInstance>();
    handle.add_class::<HelloWorld>();
    handle.add_class::<Hand>();
}

// Macro that creates the entry-points of the dynamic library.
godot_init!(init);
