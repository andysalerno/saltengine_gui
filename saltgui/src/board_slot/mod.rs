use gdnative::{api::InputEventMouseButton, prelude::*};
use log::info;

#[derive(NativeClass)]
#[inherit(Spatial)]
pub struct BoardSlot {}

impl BoardSlot {
    fn new(_owner: &Spatial) -> Self {
        Self {}
    }
}

#[methods]
impl BoardSlot {
    #[export]
    fn _ready(&self, owner: TRef<Spatial>) {
        let mouse_collider = owner.get_node("Area").unwrap();
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

        info!("BoardSlot is ready.");
    }

    #[export]
    fn input_event(
        &mut self,
        _owner: TRef<Spatial>,
        _camera: Variant,
        mouse_event: Variant,

        // Clicked position in world-space.
        _click_pos: Variant,
        _click_normal: Variant,
        _shape_idx: Variant,
    ) {
        if let Some(event) = mouse_event.try_to_object::<InputEventMouseButton>() {
            let click = unsafe { event.assume_safe() };
            if !click.is_pressed() {
                info!("Released over boardslot");
            }
        }
    }
}
