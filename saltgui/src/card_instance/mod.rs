use crate::textbox::TextBox;
use crate::util::NodeRef;
use crate::{util, SignalName};
use gdnative::api::InputEventMouseButton;
use gdnative::prelude::*;
use log::info;
use salt_engine::cards::UnitCardDefinitionView;
use salt_engine::game_state::UnitCardInstancePlayerView;

const CARD_INSTANCE_SCENE: &str = "res://card/creature_instance.tscn";
const BODY_TEXT_LABEL: &str = "CardBodyText/Viewport/GUI/Panel/RichTextLabel";
const TITLE_TEXT_LABEL: &str = "CardTitleText/Viewport/GUI/Panel/RichTextLabel";
const COST_LABEL: &str = "Cost";

pub(crate) const CARD_DRAGGED: SignalName = SignalName("card_dragged");
const INPUT_EVENT: SignalName = SignalName("input_event");

#[derive(NativeClass)]
#[register_with(Self::register)]
#[inherit(Spatial)]
pub struct CardInstance {
    title: String,
    body: String,
    state_is_following_mouse: bool,
    cost_label: NodeRef<TextBox, Spatial>,
    view: Option<UnitCardInstancePlayerView>,
}

impl CardInstance {
    pub(crate) fn new(_owner: TRef<Spatial>) -> Self {
        Self {
            title: "unset".to_string(),
            body: "unset".to_string(),
            state_is_following_mouse: false,
            cost_label: NodeRef::from_path(COST_LABEL),
            view: None,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_title(&mut self, title: impl ToString) {
        self.title = title.to_string();
    }

    pub fn set_body(&mut self, body: impl ToString) {
        self.body = body.to_string();
    }

    pub fn set_view(&mut self, view: UnitCardInstancePlayerView) {
        info!("Setting view of card to one with id: {}", view.id());
        self.view = Some(view);
    }

    pub fn expect_view(&self) -> &UnitCardInstancePlayerView {
        self.view.as_ref().unwrap()
    }

    fn follow_mouse_start(&mut self, owner: &Spatial, mouse_pos: Vector2) {
        info!("Emitting signal: PLAYER_HAND_CARD_DRAGGED (starting)");
        owner.emit_signal(
            CARD_DRAGGED,
            &[
                owner.get_path().to_variant(),
                false.to_variant(),
                mouse_pos.to_variant(),
            ],
        );
        self.state_is_following_mouse = true;
    }

    fn follow_mouse_stop(&mut self, owner: &Spatial, mouse_pos: Vector2) {
        info!("Emitting signal: PLAYER_HAND_CARD_DRAGGED (ending)");
        owner.emit_signal(
            CARD_DRAGGED,
            &[
                owner.get_path().to_variant(),
                true.to_variant(),
                mouse_pos.to_variant(),
            ],
        );
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
        let card_z = original_global_pos.z as f64;

        let updated_pos = camera.project_position(mouse_pos, f64::abs(card_z));

        let mut current = owner.global_transform();
        current.origin.x = updated_pos.x;
        current.origin.y = updated_pos.y;

        owner.set_global_transform(current);
    }

    pub(crate) fn new_instance() -> Instance<CardInstance, Unique> {
        let card_instance = util::load_scene(CARD_INSTANCE_SCENE).unwrap();
        let card_instance = util::instance_scene::<Spatial>(&card_instance);
        card_instance.cast_instance::<CardInstance>().unwrap()
    }
}

#[methods]
impl CardInstance {
    #[export]
    fn _ready(&mut self, owner: TRef<Spatial>) {
        self.cost_label.init_from_parent_ref(owner);

        let body_text = owner
            .get_node(BODY_TEXT_LABEL)
            .expect("Did not find body text.");

        let title_text = owner
            .get_node(TITLE_TEXT_LABEL)
            .expect("Did not find title text.");

        unsafe {
            body_text
                .assume_safe_if_sane()
                .expect("_body_text was not sane")
                .set("text", &self.body);

            title_text
                .assume_safe_if_sane()
                .expect("_title_text was not sane")
                .set("text", &self.title);
        }

        let cost = self
            .view
            .as_ref()
            .expect("The view should be set before _ready is invoked")
            .definition()
            .cost();

        self.cost_label
            .resolve_instance()
            .map(|a, _| {
                a.set_text(&cost.to_string());
            })
            .expect("Could not update cost label");

        let mouse_collider = owner.get_node("StaticBody").unwrap();
        let mouse_collider = unsafe { mouse_collider.assume_safe_if_sane().unwrap() };
        util::connect_signal(&*mouse_collider, INPUT_EVENT, owner, "input_event");
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
            let position = click.position();
            if click.is_pressed() {
                self.follow_mouse_start(&owner, position);
            } else {
                self.follow_mouse_stop(&owner, position);
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

        builder.add_signal(Signal {
            name: CARD_DRAGGED.as_ref(),
            args: &[
                SignalArgument {
                    name: "path",
                    default: Variant::from_str("<empty_default>"),
                    export_info: ExportInfo::new(VariantType::GodotString),
                    usage: PropertyUsage::DEFAULT,
                },
                SignalArgument {
                    name: "is_ended",
                    default: Variant::from_bool(false),
                    export_info: ExportInfo::new(VariantType::Bool),
                    usage: PropertyUsage::DEFAULT,
                },
            ],
        });
    }
}
