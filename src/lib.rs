mod agent;
mod client;
mod godot_log;
mod hello;

use gdnative::prelude::*;
use hello::HelloWorld;

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    handle.add_class::<HelloWorld>();
}

// Macro that creates the entry-points of the dynamic library.
godot_init!(init);
