use gdnative::{
    nativescript::property::{EnumHint, IntHint, StringHint},
    prelude::*,
};
use log::info;

use crate::{client, godot_log::GodotLog};

#[derive(NativeClass)]
#[register_with(Self::register)]
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
    fn _ready(&self, owner: TRef<Node>) {
        GodotLog::init();

        let shared = unsafe { owner.assume_shared() };

        std::thread::spawn(move || {
            client::run(shared).unwrap();
        });

        info!("Websocket server started on a new thread.");
        // client::run(owner).unwrap();
    }

    #[export]
    fn my_method(&self, _owner: &Node) {
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
