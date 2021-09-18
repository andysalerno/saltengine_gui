mod godot_extensions;

use crate::SignalName;
use gdnative::prelude::*;
use log::info;
use std::ops::Deref;

/// Load a scene.
pub(crate) fn load_scene(path: &str) -> Option<Ref<PackedScene, ThreadLocal>> {
    let scene = ResourceLoader::godot_singleton().load(path, "PackedScene", false)?;

    let scene = unsafe { scene.assume_thread_local() };

    scene.cast::<PackedScene>()
}

/// Instance a scene.
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

/// Connect a node to a signal on another node.
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

pub(crate) fn get_as<T, B>(
    path: impl AsRef<str>,
    owner: TRef<Node>,
) -> Option<RefInstance<T, Shared>>
where
    T: NativeClass<Base = B>,
    B: SubClass<Node>,
{
    owner
        .get_node(path)
        .map(|r| unsafe { r.assume_safe() })
        .map(|r| r.cast::<B>().unwrap())
        .map(|r| r.cast_instance::<T>().unwrap())
}

/// A longer-lived reference to a Godot Node object.
/// Can be resolved to a custom (`NativeClass`) script or a Godot object.
#[derive(Debug)]
pub(crate) struct NodeRef<T, N> {
    _phantom: std::marker::PhantomData<T>,
    _phantom_b: std::marker::PhantomData<N>,
    reference: Option<Ref<Node>>,
    path: String,
}

impl<T, N> NodeRef<T, N> {
    pub fn from_existing(path: impl AsRef<str>, reference: Ref<Node>) -> Self {
        Self {
            _phantom: std::marker::PhantomData::default(),
            _phantom_b: std::marker::PhantomData::default(),
            reference: Some(reference),
            path: path.as_ref().to_string(),
        }
    }

    pub fn from_path(path: impl AsRef<str>) -> Self {
        Self {
            _phantom: std::marker::PhantomData::default(),
            _phantom_b: std::marker::PhantomData::default(),
            reference: None,
            path: path.as_ref().to_string(),
        }
    }
}

impl<T, N> NodeRef<T, N>
where
    T: SubClass<Node>,
{
    pub fn from_parent(path: impl AsRef<str>, parent: &impl SubClass<Node>) -> Self {
        let x = parent.upcast::<Node>();
        let child = x.get_node(path.as_ref()).unwrap();

        Self::from_existing(path, child)
    }

    pub fn init_from_parent<P, R>(&mut self, parent: P)
    where
        P: AsRef<R>,
        R: SubClass<Node>,
    {
        let x = parent.as_ref().upcast::<Node>();
        let child = x.get_node(&self.path).unwrap();
        self.reference = Some(child);
        info!("NodeRef {} init complete", self.path);
    }

    pub fn resolve(&self) -> TRef<T> {
        let r = unsafe {
            self.reference
                .unwrap_or_else(|| panic!("Node ref was not initialized: {}", self.path))
                .assume_safe()
        };
        let r = r.cast::<T>().unwrap();
        r
    }

    pub fn resolve_ref(&self) -> &T {
        let r = unsafe {
            self.reference
                .unwrap_or_else(|| panic!("Node ref was not initialized: {}", self.path))
                .assume_safe()
        };
        let r = r.cast::<T>().unwrap();
        r.as_ref()
    }
}

impl<T, N> NodeRef<T, N>
where
    T: NativeClass<Base = N>,
    N: SubClass<Node>,
{
    pub fn init_from_parent_ref(&mut self, parent: TRef<impl SubClass<Node>>) {
        let parent = parent.upcast::<Node>();
        let child = parent.get_node(&self.path).unwrap();
        self.reference = Some(child);
        info!("NodeRef {} init complete", self.path);
    }

    pub fn from_parent_ref(path: impl AsRef<str>, parent: TRef<Node>) -> Self {
        let child = parent.get_node(path.as_ref()).unwrap();

        Self::from_existing(path, child)
    }

    pub fn resolve_instance(&self) -> RefInstance<T, Shared> {
        let r = unsafe {
            self.reference
                .unwrap_or_else(|| panic!("Node ref was not initialized: {}", self.path))
                .assume_safe()
        };
        let r = r.cast::<N>().unwrap();
        let r = r.cast_instance::<T>().unwrap();
        r
    }
}
