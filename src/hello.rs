use gdnative::prelude::*;
use log::info;

use crate::{client, godot_log::GodotLog};

#[derive(NativeClass)]
#[inherit(Node)]
pub struct HelloWorld;

impl HelloWorld {
    fn new(_owner: &Node) -> Self {
        Self
    }
}

#[methods]
impl HelloWorld {
    #[export]
    fn _ready(&self, _owner: &Node) {
        // The `godot_print!` macro works like `println!` but prints to the Godot-editor
        // output tab as well.
        GodotLog::init();

        client::run();
    }
}
