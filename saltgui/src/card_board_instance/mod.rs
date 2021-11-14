use crate::util::{self, NodeRef};
use gdnative::api::RichTextLabel;
use gdnative::prelude::*;
use log::info;
use salt_engine::game_state::UnitCardInstancePlayerView;

const CARD_BOARD_INSTANCE_SCENE: &str = "res://card/card_board_instance/card_board_instance.tscn";
const TITLE_PATH: &str = "Title/TitleViewport/Control/Panel/RichTextLabel";
const STATS_PATH: &str = "Stats/StatsViewport/Control/Panel/RichTextLabel";

#[derive(NativeClass)]
#[register_with(Self::register)]
#[inherit(Spatial)]
pub struct CardBoardInstance {
    title_label_init: Option<String>,
    stats_label_init: Option<String>,
    stats_label: NodeRef<RichTextLabel, Spatial>,
    title_label: NodeRef<RichTextLabel, Spatial>,
    view: Option<UnitCardInstancePlayerView>,
    target_z: f32,
    cur_direction: f32,
}

const MAX_Z: f32 = -1.;
const MIN_Z: f32 = -5.;

impl CardBoardInstance {
    pub(crate) fn new(_owner: TRef<Spatial>) -> Self {
        Self {
            stats_label: NodeRef::from_path(STATS_PATH),
            title_label: NodeRef::from_path(TITLE_PATH),
            title_label_init: None,
            stats_label_init: None,
            view: None,
            target_z: MAX_Z,
            cur_direction: 1.,
        }
    }

    pub(crate) fn set_title(&mut self, title: impl AsRef<str>) {
        if let Some(r) = self.title_label.try_resolve() {
            r.set_text(title);
        } else {
            self.title_label_init = Some(title.as_ref().to_string());
        }
    }

    pub(crate) fn set_stats(&mut self, stats: impl AsRef<str>) {
        if let Some(r) = self.stats_label.try_resolve() {
            r.set_text(stats);
        } else {
            self.stats_label_init = Some(stats.as_ref().to_string());
        }
    }

    pub(crate) fn new_instance() -> Instance<CardBoardInstance, Unique> {
        let card_instance = util::load_scene(CARD_BOARD_INSTANCE_SCENE).unwrap();
        let card_instance = util::instance_scene::<Spatial>(&card_instance);
        card_instance.cast_instance::<CardBoardInstance>().unwrap()
    }
}

#[methods]
impl CardBoardInstance {
    #[export]
    fn _ready(&mut self, owner: TRef<Spatial>) {
        info!(
            "CardBoardInstance generated under parent: {:?}",
            owner.get_path()
        );
        self.stats_label.init_from_parent(owner);
        self.title_label.init_from_parent(owner);

        if let Some(init_title) = self.title_label_init.take() {
            self.set_title(init_title);
        }

        if let Some(init_stats) = self.stats_label_init.take() {
            self.set_stats(init_stats);
        }
    }

    // #[export]
    // fn _physics_process(&mut self, owner: TRef<Spatial>, delta: f32) {
    //     let owner = owner.as_ref();

    //     let cur_z = owner.translation().z;

    //     if cur_z >= MAX_Z {
    //         self.target_z = MIN_Z;
    //         self.cur_direction = -1.;
    //     } else if cur_z <= MIN_Z {
    //         self.target_z = MAX_Z;
    //         self.cur_direction = 1.;
    //     }

    //     let increment = delta * self.cur_direction;

    //     let translation = Vector3::new(0., 0., increment);

    //     owner.translate_object_local(translation);
    // }
}
