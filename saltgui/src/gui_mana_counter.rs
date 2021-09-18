use crate::util::NodeRef;
use gdnative::{api::RichTextLabel, prelude::*};
use log::{info, warn};

const LABEL_PATH: &str = "RichTextLabel";

#[derive(NativeClass, Debug)]
#[register_with(Self::register)]
#[inherit(Control)]
pub struct ManaCounter {
    textbox: NodeRef<RichTextLabel, Control>,
    is_ready: bool,
}

impl ManaCounter {
    fn new(_owner: &Control) -> Self {
        Self {
            textbox: NodeRef::from_path(LABEL_PATH),
            is_ready: false,
        }
    }
}

#[methods]
impl ManaCounter {
    #[export]
    fn _ready(&mut self, owner: TRef<Control>) {
        self.textbox.init_from_parent(owner);
        self.is_ready = true;
    }

    pub fn set_text(&self, text: &str) {
        if self.is_ready {
            info!("Setting textbox text to: {}", text);
            self.textbox.resolve_ref().set_text(text);
        } else {
            warn!("set_text invoked when TextBox is not yet ready");
        }
    }

    pub fn get_text(&self) -> GodotString {
        if self.is_ready {
            self.textbox.resolve_ref().text()
        } else {
            "<TextBox not yet ready>".into()
        }
    }

    fn register(builder: &ClassBuilder<Self>) {
        builder
            .add_property::<GodotString>("text")
            .with_getter(|s: &Self, _| s.get_text())
            .with_setter(|s: &mut Self, _owner, value| s.set_text(&value.to_string()))
            .done();
    }
}
