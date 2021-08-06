use gdnative::prelude::*;
use godot_log::GodotLog;

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    GodotLog::init();
    handle.add_class::<Hand>();
}
// Macro that creates the entry-points of the dynamic library.
godot_init!(init);

const BODY_TEXT_LABEL: &str = "CardBodyText/Viewport/GUI/Panel/RichTextLabel";
const TITLE_TEXT_LABEL: &str = "CardTitleText/Viewport/GUI/Panel/RichTextLabel";

#[derive(NativeClass)]
#[register_with(Self::register)]
#[inherit(Spatial)]
pub struct Hand {
    title: String,
    body: String,
    state_is_following_mouse: bool,
}

impl Hand {
    fn new(_owner: &Spatial) -> Self {
        Self {
            title: "unset".to_string(),
            body: "unset".to_string(),
            state_is_following_mouse: false,
        }
    }
}

#[methods]
impl Hand {
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
    }

    #[export]
    fn add_card(&mut self, _owner: TRef<Spatial>) {}

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
