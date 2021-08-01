use gdnative::api::{InputEventMouseButton, InputEventMouseMotion};
use gdnative::core_types::vector3;
use gdnative::prelude::*;
use gdnative::{
    nativescript::property::{EnumHint, IntHint, StringHint},
    prelude::*,
};
use godot_log::GodotLog;
use log::info;

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    GodotLog::init();
    handle.add_class::<CardInstance>();
}
// Macro that creates the entry-points of the dynamic library.
godot_init!(init);

const CLICK_RAISE_DIST: f32 = 10.;

#[derive(NativeClass)]
#[register_with(Self::register)]
#[inherit(Spatial)]
pub struct CardInstance {
    title: String,
    body: String,
    state_is_following_mouse: bool,
}

impl CardInstance {
    fn new(_owner: &Spatial) -> Self {
        Self {
            title: "unset".to_string(),
            body: "unset".to_string(),
            state_is_following_mouse: false,
        }
    }

    fn follow_mouse_start(&mut self, owner: &Spatial) {
        info!("mouse pressed on card");
        self.state_is_following_mouse = true;

        let translation = Vector3::new(0., 0., CLICK_RAISE_DIST / 100.);
        owner.translate(translation);

        info!("{} started following mouse info", owner.name());
    }

    fn follow_mouse_stop(&mut self, owner: &Spatial) {
        self.state_is_following_mouse = false;
        info!("mouse not pressed on card? - info");

        let translation = Vector3::new(0., 0., -CLICK_RAISE_DIST / 100.);
        owner.translate(translation);
    }
}

#[methods]
impl CardInstance {
    #[export]
    fn _ready(&self, owner: TRef<Spatial>) {
        let _body_text = owner
            .get_node("CardBodyText/Viewport/GUI/Panel/RichTextLabel")
            .expect("Did not find body text.");

        let _title_text = owner
            .get_node("CardTitleText/Viewport/GUI/Panel/RichTextLabel")
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
    fn input_event(
        &mut self,
        owner: TRef<Spatial>,
        _camera: Variant,
        mouse_event: Variant,
        _click_pos: Variant,
        _click_normal: Variant,
        _shape_idx: Variant,
    ) {
        if let Some(event) = mouse_event.try_to_object::<InputEventMouseMotion>() {
            if self.state_is_following_mouse {
                let motion = unsafe { event.assume_safe() };
                let relative = motion.relative();

                info!("Saw motion: {:?}", relative);

                let relative_three = Vector3::new(relative.x / 100., -relative.y / 100., 0.);

                owner.translate(relative_three);
            }
        } else if let Some(event) = mouse_event.try_to_object::<InputEventMouseButton>() {
            let click = unsafe { event.assume_safe() };
            if click.is_pressed() {
                self.follow_mouse_start(&owner);
            } else {
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
