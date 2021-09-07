mod godot_extensions;

use std::ops::{Deref, Sub};

use gdnative::{
    object::{AssumeSafeLifetime, LifetimeConstraint},
    prelude::*,
    ref_kind::RefKind,
};
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

pub(crate) fn connect_signal<U: SubClass<Node>>(
    from: impl Deref<Target = U>,
    signal: SignalName,
    target: impl AsArg<Object>,
    target_method: impl Into<GodotString>,
) {
    let target_method = target_method.into();

    let node = from.upcast::<Node>();

    node.connect(signal, target, target_method, VariantArray::new_shared(), 0)
        .expect("Failed binding signal");
}

#[derive(Debug)]
pub(crate) struct NodeRef<T> {
    _phantom: std::marker::PhantomData<T>,
    reference: Ref<Node>,
    path: String,
}

impl<T> NodeRef<T>
where
    // T: GodotObject<RefKind = ManuallyManaged> + SubClass<Node>,
    T: SubClass<Node>,
{
    // pub fn from_parent(path: String, parent: &Node) -> Self {
    //     let child = parent.get_node("hi").unwrap();

    //     Self::from_existing(path, child)
    // }

    // pub fn from_parentz(path: impl AsRef<str>, parent: Ref<impl SubClass<Node>>) -> Self {
    //     //let x = parent.upcast::<Node>();
    //     let child = parent.get_node(path.as_ref()).unwrap();

    //     Self::from_existing(path, child)
    // }

    pub fn from_parent(path: impl AsRef<str>, parent: &impl SubClass<Node>) -> Self {
        let x = parent.upcast::<Node>();
        let child = x.get_node(path.as_ref()).unwrap();

        Self::from_existing(path, child)
    }

    pub fn resolve(&self) -> TRef<T> {
        let r = unsafe { self.reference.assume_safe() };
        let r = r.cast::<T>().unwrap();
        r
    }

    pub fn resolve_ref(&self) -> &T {
        let r = unsafe { self.reference.assume_safe() };
        let r = r.cast::<T>().unwrap();
        r.as_ref()
    }
}

impl<T> NodeRef<T> {
    pub fn from_existing(path: impl AsRef<str>, reference: Ref<Node>) -> Self {
        Self {
            _phantom: std::marker::PhantomData::default(),
            reference,
            path: path.as_ref().to_string(),
        }
    }
}

impl<T> NodeRef<T>
where
    // T: NativeClass<Base = Node>,
    T: NativeClass<Base = Spatial>,
{
    pub fn from_parent_ref(path: impl AsRef<str>, parent: TRef<Node>) -> Self {
        let child = parent.get_node(path.as_ref()).unwrap();

        Self::from_existing(path, child)
    }

    pub fn resolve_instance(&self) -> RefInstance<T, Shared> {
        let r = unsafe { self.reference.assume_safe() };
        let r = r.cast::<Spatial>().unwrap();
        let r = r.cast_instance::<T>().unwrap();
        r
    }
}
