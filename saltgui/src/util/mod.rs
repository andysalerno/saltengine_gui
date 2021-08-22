mod godot_extensions;

use std::ops::Deref;

use gdnative::prelude::*;
use log::info;

use crate::SignalName;

pub(crate) fn load_scene(path: &str) -> Option<Ref<PackedScene, ThreadLocal>> {
    let scene = ResourceLoader::godot_singleton().load(path, "PackedScene", false)?;

    let scene = unsafe { scene.assume_thread_local() };

    scene.cast::<PackedScene>()
}

pub(crate) fn instance_scene<TRoot>(scene: &PackedScene) -> Ref<TRoot, Unique>
where
    TRoot: gdnative::GodotObject<RefKind = ManuallyManaged> + SubClass<Node>,
{
    let instance = scene
        .instance(PackedScene::GEN_EDIT_STATE_DISABLED)
        .unwrap();

    let instance = unsafe { instance.assume_unique() };

    instance.try_cast::<TRoot>().unwrap()
}

pub(crate) fn connect_signal<T: SubClass<Node>>(
    from: &impl SubClass<Node>,
    signal: SignalName,
    target: TRef<T>,
    target_method: impl Into<GodotString>,
) {
    let object = from.upcast::<Node>();
    let target = target.upcast::<Node>();

    let target_method = target_method.into();

    object
        .connect(signal, target, target_method, VariantArray::new_shared(), 0)
        .expect("Failed binding signal");

    // info!(
    //     "Bound signal {} from {:?} to {:?}:{}",
    //     signal,
    //     object.get_path(),
    //     target.get_path(),
    //     target_method.to_string()
    // );
}
