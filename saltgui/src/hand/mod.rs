use gdnative::prelude::*;

use log::info;
use salt_engine::{cards::UnitCardDefinitionView, game_state::UnitCardInstancePlayerView};

use crate::{
    card_instance::{CardInstance, CARD_DRAGGED},
    util, SignalName,
};

const OFFSET_DIST_MULTIPLIER: f32 = 1.75;
pub(crate) const PLAYER_HAND_CARD_ADDED_SIGNAL: SignalName =
    SignalName("card_added_to_player_hand");

pub(crate) const PLAYER_HAND_CARD_DRAGGED: SignalName = SignalName("player_hand_card_dragged");

#[derive(NativeClass)]
#[register_with(Self::register)]
#[inherit(Spatial)]
pub struct Hand {
    hand_len: i32,
}

impl Hand {
    fn new(_owner: &Spatial) -> Self {
        Self { hand_len: 0 }
    }
}

#[methods]
impl Hand {
    #[export]
    fn _ready(&self, _owner: TRef<Spatial>) {
        info!("Hand is ready.");
    }

    #[export]
    fn add_card(&mut self, _owner: TRef<Spatial>) {
        info!("add_card was invoked");
    }

    fn register(builder: &ClassBuilder<Self>) {
        builder
            .add_property::<i32>("hand_len")
            .with_getter(|s: &Self, _| s.hand_len)
            .with_setter(|s, _, value| s.hand_len = value)
            .done();

        builder.add_signal(Signal {
            name: PLAYER_HAND_CARD_ADDED_SIGNAL.as_ref(),
            args: &[SignalArgument {
                name: "path",
                default: Variant::from_str("<empty_default>"),
                export_info: ExportInfo::new(VariantType::GodotString),
                usage: PropertyUsage::DEFAULT,
            }],
        });

        builder.add_signal(Signal {
            name: PLAYER_HAND_CARD_DRAGGED.as_ref(),
            args: &[
                SignalArgument {
                    name: "path",
                    default: Variant::from_str("<empty_default>"),
                    export_info: ExportInfo::new(VariantType::GodotString),
                    usage: PropertyUsage::DEFAULT,
                },
                SignalArgument {
                    name: "is_ended",
                    default: Variant::from_bool(false),
                    export_info: ExportInfo::new(VariantType::Bool),
                    usage: PropertyUsage::DEFAULT,
                },
            ],
        });
    }

    #[export]
    fn on_card_dragged(
        &self,
        owner: TRef<Spatial>,
        dragged_card_path: Variant,
        is_ended: Variant,
        mouse_pos_2d: Variant,
    ) {
        info!(
            "Hand saw card dragged signal: {:?} is ended: {}",
            dragged_card_path,
            is_ended.to_bool()
        );

        owner.emit_signal(
            PLAYER_HAND_CARD_DRAGGED,
            &[dragged_card_path, is_ended, mouse_pos_2d],
        );
    }
}

pub struct HandRef<'a> {
    node: TRef<'a, Spatial>,
}

impl<'a> HandRef<'a> {
    pub fn new(node: TRef<'a, Spatial>) -> Self {
        Self { node }
    }

    pub fn add_card(&mut self, card: &UnitCardInstancePlayerView) {
        info!("Hand is receiving a card: {}", card.definition().title());

        let card_instance = CardInstance::new_instance();

        let hand = self.node.cast_instance::<Hand>().unwrap();
        let offset = hand.map(|n, _| n.hand_len).unwrap() as f32 * OFFSET_DIST_MULTIPLIER;

        card_instance
            .map_mut(|c, n| {
                let def = card.definition();
                c.set_title(def.title());
                c.set_body(def.text());

                c.set_view(card.clone());

                n.translate(Vector3::new(offset, 0., 0.));

                util::connect_signal(n, CARD_DRAGGED, self.node, "on_card_dragged");
            })
            .unwrap();

        hand.map_mut(|hand, _| hand.hand_len += 1).unwrap();

        // let card_instance = card_instance.into_base().into_shared();
        let card_instance = card_instance.into_base();
        let card_instance = card_instance.into_shared();
        self.node.add_child(card_instance, false);

        let card_instance = unsafe { card_instance.assume_safe() };
        let card_path = card_instance.get_path();

        // what happens if we try getting it now?

        // try doing it after adding to scene??
        // let card_instance = card_instance
        //     .cast_instance::<CardInstance>()
        //     .expect("Could not cast to CardInstance.");

        //card_instance.map(|a, b| {});
        // let card_cloned = card.clone();
        // let node_cloned = self.node;

        // card_instance
        //     .map_mut(move |c, n| {
        //         info!("Start card_instance map_mut.");
        //         let def = card_cloned.definition();
        //         c.set_title(def.title());
        //         c.set_body(def.text());

        //         // c.set_view(card.clone());

        //         n.translate(Vector3::new(offset, 0., 0.));

        //         util::connect_signal(n, CARD_DRAGGED, node_cloned, "on_card_dragged");
        //         info!("Done mapping signal in card instance map_mut.");
        //     })
        //     .unwrap();

        self.node
            .emit_signal(PLAYER_HAND_CARD_ADDED_SIGNAL, &[card_path.to_variant()]);

        info!("Added card {:?} to PlayerHand.", card_path);
    }

    // fn center_hand(&self) {
    //     let current_width = self.node.
    //     let current_midpoint = self.node.translation().x;
    // }
}
