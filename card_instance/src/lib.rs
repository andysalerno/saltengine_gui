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
#[inherit(Node)]
pub struct CardInstance {
    title: String,
    body: String,
}

impl CardInstance {
    fn new(_owner: &Node) -> Self {
        Self {
            title: "unset".to_string(),
            body: "unset".to_string(),
        }
    }
}

#[methods]
impl CardInstance {
    #[export]
    fn _ready(&self, owner: TRef<Node>) {
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
    }

    fn register(builder: &ClassBuilder<Self>) {
        // no-op currently
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
