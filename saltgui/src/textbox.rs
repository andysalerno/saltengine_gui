use crate::{
    board_slot::INPUT_EVENT_SIGNAL,
    util::{self, NodeRef},
    SignalName,
};
use gdnative::{
    api::{InputEventMouseButton, RichTextLabel},
    prelude::*,
};
use log::info;

const LABEL_PATH: &str = "CardTitleText/Viewport/GUI/Panel/RichTextLabel";

#[derive(NativeClass)]
#[register_with(Self::register)]
#[inherit(Spatial)]
pub(crate) struct TextBox {
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

        info!("Textbox generated.");
    }
    fn register(builder: &ClassBuilder<Self>) {
        // builder
        //     .add_property::<String>("title")
        //     .with_getter(|s: &Self, _| s.title.clone())
        //     .with_setter(|s: &mut Self, _owner, value| {
        //         s.title = value;
        //     })
        //     .done();

        // builder
        //     .add_property::<String>("body")
        //     .with_getter(|s: &Self, _| s.body.clone())
        //     .with_setter(|s: &mut Self, _owner, value| {
        //         s.body = value;
        //     })
        //     .done();
    }
}
