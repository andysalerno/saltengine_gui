use std::thread::JoinHandle;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use gdnative::{
    nativescript::property::{EnumHint, IntHint, StringHint},
    prelude::*,
};
use log::info;

use crate::{client, godot_log::GodotLog};

#[derive(NativeClass)]
#[register_with(Self::register)]
#[inherit(Node)]
pub struct HelloWorld {
    recv: Receiver<String>,
    network_thread: JoinHandle<()>,
}

impl HelloWorld {
    fn new(_owner: &Node) -> Self {
        let (s, r) = unbounded();

        let handle = std::thread::spawn(move || {
            client::run(s).unwrap();
        });

        info!("Websocket server started on a new thread.");

        Self {
            recv: r,
            network_thread: handle,
        }
    }
}

#[methods]
impl HelloWorld {
    #[export]
    fn _ready(&self, _owner: TRef<Node>) {
        GodotLog::init();
    }

    #[export]
    fn _process(&self, _owner: TRef<Node>, _delta: f64) {
        match self.recv.try_recv() {
            Ok(msg) => godot_print!("Saw message: {}", msg),
            Err(TryRecvError::Disconnected) => godot_print!("Disconnected"),
            _ => {}
        }
    }

    #[export]
    fn my_method(&self, _owner: TRef<Node>) {
        info!("Invoked my_method.");
    }

    fn register(builder: &ClassBuilder<Self>) {
        builder
            .add_property::<String>("test/test_enum")
            .with_hint(StringHint::Enum(EnumHint::new(vec![
                "Hello".into(),
                "World".into(),
                "Testing".into(),
            ])))
            .with_getter(|_: &HelloWorld, _| "Hello".to_string())
            .done();

        builder
            .add_property("test/test_flags")
            .with_hint(IntHint::Flags(EnumHint::new(vec![
                "A".into(),
                "B".into(),
                "C".into(),
                "D".into(),
            ])))
            .with_getter(|_: &HelloWorld, _| 0)
            .done();

        builder.add_signal(Signal {
            name: "ws_message_received",
            args: &[SignalArgument {
                name: "message",
                default: Variant::from_str("<empty_default>"),
                export_info: ExportInfo::new(VariantType::GodotString),
                usage: PropertyUsage::DEFAULT,
            }],
        });
    }
}
