use gdnative::api::{Camera, InputEventMouseButton, InputEventMouseMotion};
use gdnative::prelude::*;
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
    fn new(_owner: &Spatial) -> Self {
        Self {
            title: "unset".to_string(),
            body: "unset".to_string(),
            state_is_following_mouse: false,
        }
    }

    fn follow_mouse_start(&mut self, owner: &Spatial) {
        self.state_is_following_mouse = true;

        // let translation = Vector3::new(0., 0., CLICK_RAISE_DIST / 100.);
        // owner.translate(translation);
    }

    fn follow_mouse_stop(&mut self, owner: &Spatial) {
        self.state_is_following_mouse = false;

        // let translation = Vector3::new(0., 0., -CLICK_RAISE_DIST / 100.);
        // owner.translate(translation);
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
    }

    #[export]
    fn input_event(
        &mut self,
        owner: TRef<Spatial>,
        _camera: Variant,
        mouse_event: Variant,

        // Clicked position in world-space.
        click_pos: Variant,
        _click_normal: Variant,
        _shape_idx: Variant,
    ) {
        if let Some(_event) = mouse_event.try_to_object::<InputEventMouseMotion>() {
            if self.state_is_following_mouse {
                // {
                //     let click_pos = click_pos.try_to_vector3().unwrap();
                //     info!("-- click pos: {:?}", click_pos);

                //     let original_pos = owner.translation();
                //     let original_global = owner.to_global(original_pos);

                //     info!("original local: {:?}", original_pos);
                //     info!("original global: {:?}", original_global);

                //     let click_local = owner.to_local(click_pos);
                //     info!("click local: {:?}", click_local);

                //     //let next_local = Vector3::new(click_local.x, click_local.y, original_pos.z);
                //     let next_local = Vector3::new(click_pos.x, click_pos.y, original_pos.z);
                //     info!("next local: {:?}", next_local);

                //     owner.set_translation(next_local);
                // }
                // {
                //     let click_pos = click_pos.try_to_vector3().unwrap();
                //     info!("-- click pos: {:?}", click_pos);

                //     let original_pos = owner.translation();
                //     let original_global = owner.to_global(original_pos);

                //     info!("original local: {:?}", original_pos);
                //     info!("original global: {:?}", original_global);

                //     let next_local = Vector3::new(click_pos.x, click_pos.y, original_pos.z);
                //     info!("next local: {:?}", next_local);

                //     let mut diff = click_pos - original_global;
                //     diff.z = 0.;
                //     owner.global_translate(diff);
                // }
            }
        } else if let Some(event) = mouse_event.try_to_object::<InputEventMouseButton>() {
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
