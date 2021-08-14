use super::gui_message::GuiMessage;
use crate::{
    agent::client,
    card_instance::CardInstance,
    hand::{Hand, HandRef},
    util,
};
use crossbeam::channel::{unbounded, Receiver, TryRecvError};
use gdnative::{
    nativescript::property::{EnumHint, IntHint, StringHint},
    prelude::*,
};
use godot_log::GodotLog;
use log::info;
use salt_engine::{
    cards::UnitCardDefinitionView,
    game_state::{
        board::BoardView, GameStatePlayerView, GameStateView, HandView, IterAddons,
        UnitCardInstancePlayerView,
    },
};
use std::thread::JoinHandle;

const CREATURE_INSTANCE_SCENE: &str = "res://card/creature_instance.tscn";
const PLAYER_HAND_NAME: &str = "PlayerHand";

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
        info!("Updating from state.");
        let hand_ref = owner.get_node(PLAYER_HAND_NAME).unwrap();
        let hand_ref = unsafe { hand_ref.assume_safe() };
        let hand_ref = hand_ref.cast::<Spatial>().unwrap();
        let mut hand_ref = HandRef::new(hand_ref);
        for hand_card in state.hand().cards() {
            info!("iterating over hand_card...");
            hand_ref.add_card(hand_card);
        }
    }

    fn add_card_instance(&self, card_view: &UnitCardInstancePlayerView, owner: TRef<Node>) {
        let creature_instance = util::load_scene(CREATURE_INSTANCE_SCENE).unwrap();
        let creature_instance = util::instance_scene::<Spatial>(&creature_instance);

        creature_instance.set("title", card_view.definition().title());
        creature_instance.set("body", card_view.definition().text());

        let mut current_translation = creature_instance.translation();
        current_translation.z = -4.5;
        creature_instance.set_translation(current_translation);

        owner.add_child(creature_instance, false);
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
        // builder
        //     .add_property::<String>("test/test_enum")
        //     .with_hint(StringHint::Enum(EnumHint::new(vec![
        //         "Hello".into(),
        //         "World".into(),
        //         "Testing".into(),
        //     ])))
        //     .with_getter(|_: &HelloWorld, _| "Hello".to_string())
        //     .done();

        // builder
        //     .add_property("test/test_flags")
        //     .with_hint(IntHint::Flags(EnumHint::new(vec![
        //         "A".into(),
        //         "B".into(),
        //         "C".into(),
        //         "D".into(),
        //     ])))
        //     .with_getter(|_: &HelloWorld, _| 0)
        //     .done();

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
