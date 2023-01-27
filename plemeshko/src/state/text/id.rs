use serde::Deserialize;

use crate::state::{components::ComponentId, label_factory::LabelFactory};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
pub struct TextId(pub(super) String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FatTextId(pub ComponentId, pub TextId);

#[derive(Default)]
#[repr(transparent)]
pub struct TextIdFactory(LabelFactory);

pub trait TextIdentifier {
    fn as_text_id(&self) -> &str;
}

impl TextId {
    pub fn in_component(self, component_id: ComponentId) -> FatTextId {
        FatTextId(component_id, self)
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
        self.0.with_lock(|lf| unsafe { f(std::mem::transmute(lf)) })
    }

    /// See [`LabelFactory::branch`].
    pub fn branch(self, name: &str) -> Self {
        TextIdFactory(self.0.branch(name))
    }

    /// See [`LabelFactory::with_branch`].
    pub fn with_branch<R>(&mut self, name: &str, f: impl FnOnce(&mut TextIdFactory) -> R) -> R {
        self.0
            .with_branch(name, |lf| unsafe { f(std::mem::transmute(lf)) })
    }

    /// See [`LabelFactory::create`].
    pub fn create(&self, name: &str) -> TextId {
        TextId(self.0.create(name))
    }
}

impl TextIdentifier for str {
    fn as_text_id(&self) -> &str {
        self
    }
}

impl TextIdentifier for TextId {
    fn as_text_id(&self) -> &str {
        self.0.as_str()
    }
}
