use crate::util::NodeRef;
use gdnative::{api::RichTextLabel, prelude::*};

const LABEL_PATH: &str = "Viewport/GUI/Panel/RichTextLabel";

#[derive(NativeClass, Debug)]
#[register_with(Self::register)]
#[inherit(Spatial)]
pub struct TextBox {
    textbox: Option<NodeRef<RichTextLabel>>,
}

impl TextBox {
    fn new(_owner: &Spatial) -> Self {
        Self { textbox: None }
    }
}

#[methods]
impl TextBox {
    #[export]
    fn _ready(&mut self, owner: TRef<Spatial>) {
        let r: NodeRef<RichTextLabel> = NodeRef::from_parent(LABEL_PATH, owner.as_ref());

        self.textbox = Some(r);
    }

    pub fn set_text(&self, text: &str) {
        if let Some(textbox) = &self.textbox {
            let x = textbox.resolve_ref();
            x.set_text(text);
        }
    }

    pub fn get_text(&self) -> GodotString {
        self.textbox.as_ref().map_or_else(
            || String::new().into(),
            |textbox| {
                let x = textbox.resolve_ref();
                x.text()
            },
        )
    }

    fn register(builder: &ClassBuilder<Self>) {
        builder
            .add_property::<GodotString>("text")
            .with_getter(|s: &Self, _| s.get_text())
            .with_setter(|s: &mut Self, _owner, value| s.set_text(&value.to_string()))
            .done();
    }
}
