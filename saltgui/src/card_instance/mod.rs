use gdnative::api::InputEventMouseButton;
use gdnative::prelude::*;

use crate::util;

const CARD_INSTANCE_SCENE: &str = "res://card/creature_instance.tscn";
const BODY_TEXT_LABEL: &str = "CardBodyText/Viewport/GUI/Panel/RichTextLabel";
const TITLE_TEXT_LABEL: &str = "CardTitleText/Viewport/GUI/Panel/RichTextLabel";

#[derive(NativeClass)]
#[register_with(Self::register)]
#[inherit(Spatial)]
pub struct CardInstance {
    title: String,
    body: String,
    state_is_following_mouse: bool,
}

impl CardInstance {
    pub(crate) fn new(_owner: &Spatial) -> Self {
        Self {
            title: "unset".to_string(),
            body: "unset".to_string(),
            state_is_following_mouse: false,
        }
    }

    pub fn set_title(&mut self, title: impl ToString) {
        self.title = title.to_string();
    }

    pub fn set_body(&mut self, body: impl ToString) {
        self.body = body.to_string();
    }

    fn follow_mouse_start(&mut self, _owner: &Spatial) {
        self.state_is_following_mouse = true;
    }

    fn follow_mouse_stop(&mut self, _owner: &Spatial) {
        self.state_is_following_mouse = false;
    }

    fn follow_mouse_update(&self, owner: &Spatial) {
        let tree = owner.get_tree().unwrap();
        let tree = unsafe { tree.assume_safe() };
        let root = tree.root().unwrap();
        let root = unsafe { root.assume_safe() };

        let camera = root.get_camera().unwrap();
        let camera = unsafe { camera.assume_safe() };

        let mouse_pos = root.get_mouse_position();
        let original_global_pos = owner.global_transform().origin;
        let card_z = original_global_pos.z;

        let updated_pos = camera.project_position(mouse_pos, f32::abs(card_z) as f64);

        let mut current = owner.global_transform();
        current.origin.x = updated_pos.x;
        current.origin.y = updated_pos.y;

        owner.set_global_transform(current);
    }

    pub(crate) fn new_instance() -> Instance<CardInstance, Unique> {
        let card_instance = util::load_scene(CARD_INSTANCE_SCENE).unwrap();
        let card_instance = util::instance_scene::<Spatial>(&card_instance);
        let card_instance = card_instance.cast_instance::<CardInstance>().unwrap();

        card_instance
    }
}

#[methods]
impl CardInstance {
    #[export]
    fn _ready(&self, owner: TRef<Spatial>) {
        let _body_text = owner
            .get_node(BODY_TEXT_LABEL)
            .expect("Did not find body text.");

        let _title_text = owner
            .get_node(TITLE_TEXT_LABEL)
            .expect("Did not find title text.");

        unsafe {
            _body_text
                .assume_safe_if_sane()
                .expect("_body_text was not sane")
                .set("text", &self.body);

            _title_text
                .assume_safe_if_sane()
                .expect("_title_text was not sane")
                .set("text", &self.title);
        }

        let mouse_collider = owner.get_node("StaticBody").unwrap();
        let mouse_collider = unsafe { mouse_collider.assume_safe_if_sane().unwrap() };
        mouse_collider
            .connect(
                "input_event",
                owner,
                "input_event",
                VariantArray::new_shared(),
                0,
            )
            .expect("failed to connect signal");
    }

    #[export]
    fn _physics_process(&mut self, owner: TRef<Spatial>, _delta: f32) {
        if self.state_is_following_mouse {
            self.follow_mouse_update(owner.as_ref());
        }
    }

    #[export]
    fn input_event(
        &mut self,
        owner: TRef<Spatial>,
        _camera: Variant,
        mouse_event: Variant,

        // Clicked position in world-space.
        _click_pos: Variant,
        _click_normal: Variant,
        _shape_idx: Variant,
    ) {
        if let Some(event) = mouse_event.try_to_object::<InputEventMouseButton>() {
            let click = unsafe { event.assume_safe() };
            if click.is_pressed() {
                // info!("start following");
                self.follow_mouse_start(&owner);
            } else {
                // info!("stop following");
                self.follow_mouse_stop(&owner);
            }
        }
    }

    fn register(builder: &ClassBuilder<Self>) {
        builder
            .add_property::<String>("title")
            .with_getter(|s: &Self, _| s.title.clone())
            .with_setter(|s: &mut Self, _owner, value| {
                s.title = value;
            })
            .done();

        builder
            .add_property::<String>("body")
            .with_getter(|s: &Self, _| s.body.clone())
            .with_setter(|s: &mut Self, _owner, value| {
                s.body = value;
            })
            .done();
    }
}
