use gdnative::prelude::*;

use log::info;
use salt_engine::{cards::UnitCardDefinitionView, game_state::UnitCardInstancePlayerView};

use crate::card_instance::CardInstance;

const OFFSET_DIST_MULTIPLIER: f32 = 1.75;

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
        let card_instance = CardInstance::new_instance();
        let (card_instance, _) = card_instance.decouple();
        let card_instance = card_instance.into_shared();
        let card_instance = unsafe { card_instance.assume_safe() };

        card_instance.set("title", card.definition().title());
        card_instance.set("body", card.definition().text());

        let hand = self.node.cast_instance::<Hand>().unwrap();
        let offset = hand.map(|n, _| n.hand_len).unwrap() as f32 * OFFSET_DIST_MULTIPLIER;

        card_instance.translate(Vector3::new(offset, 0., 0.));

        hand.map_mut(|hand, _| hand.hand_len += 1).unwrap();

        info!(
            "Transform before: {:?}, translation before: {:?}",
            self.node.transform(),
            self.node.translation()
        );
        self.node.add_child(card_instance, false);
        info!(
            "Transform after: {:?}, translation before: {:?}",
            self.node.transform(),
            self.node.translation()
        );

        info!("Added child.");
    }

    // fn center_hand(&self) {
    //     let current_width = self.node.
    //     let current_midpoint = self.node.translation().x;
    // }
}
