use crate::{
    card_instance::{CardInstance, CARD_DRAGGED},
    util, SignalName,
};
use gdnative::prelude::*;
use log::info;
use salt_engine::{cards::UnitCardDefinitionView, game_state::UnitCardInstancePlayerView};

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

    pub fn add_card(&mut self, card: &UnitCardInstancePlayerView, owner: TRef<Spatial>) {
        info!("Hand is receiving a card: {}", card.definition().title());

        let card_instance = CardInstance::new_instance();

        // let offset = hand.map(|n, _| n.hand_len).unwrap() as f32 * OFFSET_DIST_MULTIPLIER;
        let offset = self.hand_len as f32 * OFFSET_DIST_MULTIPLIER;

        card_instance
            .map_mut(|c, n| {
                let def = card.definition();
                c.set_title(def.title());
                c.set_body(def.text());

                c.set_view(card.clone());

                n.translate(Vector3::new(offset, 0., 0.));

                util::connect_signal(n, CARD_DRAGGED, owner, "on_card_dragged");
            })
            .unwrap();

        self.hand_len += 1;

        let card_instance = card_instance.into_base();
        let card_instance = card_instance.into_shared();
        owner.add_child(card_instance, false);

        let card_instance = unsafe { card_instance.assume_safe() };
        let card_path = card_instance.get_path();

        owner.emit_signal(PLAYER_HAND_CARD_ADDED_SIGNAL, &[card_path.to_variant()]);

        info!("Added card {:?} to PlayerHand.", card_path);
    }
}

#[methods]
impl Hand {
    #[export]
    fn _ready(&self, _owner: TRef<Spatial>) {
        info!("Hand is ready.");
    }

    // #[export]
    // fn add_card(&mut self, _owner: TRef<Spatial>) {
    //     info!("add_card was invoked");
    // }

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
