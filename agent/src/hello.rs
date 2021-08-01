use std::thread::JoinHandle;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use gdnative::{
    nativescript::property::{EnumHint, IntHint, StringHint},
    prelude::*,
};
use godot_log::GodotLog;
use log::info;
use salt_engine::{
    cards::UnitCardDefinitionView,
    game_state::{board::BoardView, GameStatePlayerView, IterAddons, UnitCardInstancePlayerView},
};

use crate::{client, gui_message::GuiMessage};

const CREATURE_INSTANCE_SCENE: &str = "res://card/creature_instance.tscn";

#[derive(NativeClass)]
#[register_with(Self::register)]
#[inherit(Node)]
pub struct HelloWorld {
    recv: Receiver<GuiMessage>,
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

    fn update_from_state(&self, state: GameStatePlayerView, owner: TRef<Node>) {
        //owner.get_
        //TRef::fr
        for creature in state.board().slots_iter().exclude_heroes().creatures() {
            self.add_card_instance(creature, owner);
        }
        // for creature in state.board().slots_iter().exclude_heroes().with_creature() {}
    }

    fn add_card_instance(&self, card_view: &UnitCardInstancePlayerView, owner: TRef<Node>) {
        let creature_instance = Self::load_scene(CREATURE_INSTANCE_SCENE).unwrap();
        let creature_instance = Self::instance_scene::<Spatial>(&creature_instance);

        creature_instance.set("title", card_view.definition().title());
        creature_instance.set("body", card_view.definition().text());
        // creature_instance.set_name("testingtesting");

        // let temp_node = unsafe { owner.assume_safe_if_sane().expect("root node not sane") };
        //temp_node.add_child(creature_instance.into_shared(), false);
        owner.add_child(creature_instance, false);
    }

    fn load_scene(path: &str) -> Option<Ref<PackedScene, ThreadLocal>> {
        let scene = ResourceLoader::godot_singleton().load(path, "PackedScene", false)?;

        let scene = unsafe { scene.assume_thread_local() };

        scene.cast::<PackedScene>()
    }

    /// Root here is needs to be the same type (or a parent type) of the node that you put in the child
    ///   scene as the root. For instance Spatial is used for this example.
    fn instance_scene<TRoot>(scene: &PackedScene) -> Ref<TRoot, Unique>
    where
        TRoot: gdnative::GodotObject<RefKind = ManuallyManaged> + SubClass<Node>,
    {
        let instance = scene
            .instance(PackedScene::GEN_EDIT_STATE_DISABLED)
            .unwrap();

        let instance = unsafe { instance.assume_unique() };

        instance.try_cast::<TRoot>().unwrap()
    }
}

#[methods]
impl HelloWorld {
    #[export]
    fn _ready(&self, _owner: TRef<Node>) {
        GodotLog::init();
    }

    #[export]
    fn _process(&self, owner: TRef<Node>, _delta: f64) {
        let message = match self.recv.try_recv() {
            Ok(msg) => msg,
            Err(TryRecvError::Disconnected) => return, // todo: display something?
            _ => return,
        };

        match message {
            GuiMessage::StateUpdate(state) => self.update_from_state(state, owner),
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
