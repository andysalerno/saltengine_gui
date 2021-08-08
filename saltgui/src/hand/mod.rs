use gdnative::prelude::*;
use godot_log::GodotLog;
use log::info;
use salt_engine::game_state::UnitCardInstancePlayerView;

use crate::{card_instance::CardInstance, util};

const CARD_INSTANCE_SCENE: &str = "res://card/creature_instance.tscn";
const BODY_TEXT_LABEL: &str = "CardBodyText/Viewport/GUI/Panel/RichTextLabel";
const TITLE_TEXT_LABEL: &str = "CardTitleText/Viewport/GUI/Panel/RichTextLabel";

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
    fn _ready(&self, owner: TRef<Spatial>) {
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

        let hand = self.node.cast_instance::<Hand>().unwrap();
        let offset = hand.map(|n, _| n.hand_len).unwrap();

        card_instance.translate(Vector3::new(offset as f32, 0., 0.));

        hand.map_mut(|hand, _| hand.hand_len += 1).unwrap();

        self.node.add_child(card_instance, false);

        info!("Added child.");
    }
}
