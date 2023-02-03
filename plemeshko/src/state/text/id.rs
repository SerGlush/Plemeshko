use std::borrow::Borrow;

use bytemuck::TransparentWrapper;
use serde::Deserialize;

use crate::state::{components::ComponentId, label_factory::LabelFactory};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
pub struct TextId(pub(super) String);

#[repr(transparent)]
pub struct TextIdRef(pub(super) str);

// SAFETY:
// Has `#[repr(transparent)]` and only 1 field.
unsafe impl TransparentWrapper<str> for TextIdRef {}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FatTextId(pub ComponentId, pub TextId);

#[derive(Default)]
#[repr(transparent)]
pub struct TextIdFactory(LabelFactory);

// SAFETY:
// Has `#[repr(transparent)]` and only 1 field.
unsafe impl TransparentWrapper<LabelFactory> for TextIdFactory {}

impl TextId {
    pub fn in_component(self, component_id: ComponentId) -> FatTextId {
        FatTextId(component_id, self)
    }
}

impl TextIdRef {
    pub(super) fn report(&self) -> String {
        self.0.to_owned()
    }

    pub fn from_str(value: &str) -> &Self {
        TextIdRef::wrap_ref(value)
    }
}

impl TextIdFactory {
    pub fn new() -> Self {
        TextIdFactory::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        TextIdFactory(LabelFactory::with_capacity(capacity))
    }

    /// See [`LabelFactory::with_lock`].
    pub fn with_lock<R>(&mut self, f: impl FnOnce(&mut TextIdFactory) -> R) -> R {
        self.0.with_lock(|lf| f(TextIdFactory::wrap_mut(lf)))
    }

    /// See [`LabelFactory::branch`].
    pub fn branch(self, name: &str) -> Self {
        TextIdFactory(self.0.branch(name))
    }

    /// See [`LabelFactory::with_branch`].
    pub fn with_branch<R>(&mut self, name: &str, f: impl FnOnce(&mut TextIdFactory) -> R) -> R {
        self.0
            .with_branch(name, |lf| f(TextIdFactory::wrap_mut(lf)))
    }

    /// See [`LabelFactory::create`].
    pub fn create(&self, name: &str) -> TextId {
        TextId(self.0.create(name))
    }
}

impl Borrow<TextIdRef> for TextId {
    fn borrow(&self) -> &TextIdRef {
        TextIdRef::from_str(self.0.as_str())
    }
}
