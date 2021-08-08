use gdnative::prelude::*;

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
