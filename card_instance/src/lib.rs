use gdnative::api::InputEventMouseMotion;
use gdnative::core_types::vector3;
use gdnative::prelude::*;
use gdnative::{
    nativescript::property::{EnumHint, IntHint, StringHint},
    prelude::*,
};

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    // handle.add_class::<HelloWorld>();
    handle.add_class::<CardInstance>();
}
// Macro that creates the entry-points of the dynamic library.
godot_init!(init);

#[derive(NativeClass)]
#[register_with(Self::register)]
#[inherit(Spatial)]
pub struct CardInstance {
    title: String,
    body: String,
}

impl CardInstance {
    fn new(_owner: &Spatial) -> Self {
        Self {
            title: "unset".to_string(),
            body: "unset".to_string(),
        }
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
        &self,
        _owner: TRef<Spatial>,
        camera: Variant,
        input_event_mouse_motion: Variant,
        click_pos: Variant,
        click_normal: Variant,
        shape_idx: Variant,
    ) {
        let motion = input_event_mouse_motion
            .try_to_object::<InputEventMouseMotion>()
            .unwrap();
        // let motion = unsafe { motion.assume_safe() };
        let motion = unsafe { motion.assume_thread_local() };
        let relative = motion.relative();

        godot_print!("Saw motion: {:?}", relative);

        let relative_three = Vector3::new(relative.x / 100., -relative.y / 100., 0.);

        _owner.translate(relative_three);
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
