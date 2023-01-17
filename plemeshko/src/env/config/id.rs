use std::{
    fmt::{Display, Formatter},
    marker::PhantomData,
};

use educe::Educe;
use serde::{Deserialize, Serialize};

use super::{Config, Serializable};

#[derive(Educe, Serialize, Deserialize)]
#[educe(Hash, PartialEq, Eq, Debug, Clone)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ConfigLabel<C>(pub(super) String, pub(super) PhantomData<C>);

// todo: check if reexport doesn't break pub(super)
pub(super) type UnderId = u64;

#[derive(Educe)]
#[educe(Hash, PartialEq, Eq, Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ConfigId<C>(pub(super) UnderId, pub(super) PhantomData<C>);

impl<C> ConfigLabel<C> {
    pub(super) fn new(label: String) -> Self {
        ConfigLabel(label, PhantomData)
    }
}

impl<C> ConfigId<C> {
    pub(super) fn new(id: UnderId) -> Self {
        ConfigId(id, PhantomData)
    }
}

impl<C> From<String> for ConfigLabel<C> {
    fn from(value: String) -> Self {
        ConfigLabel(value, PhantomData)
    }
}

impl<C: Config> Serializable for ConfigId<C> {
    type Raw = ConfigLabel<C>;

    fn from_serializable(raw: Self::Raw, indexer: &mut super::ConfigIndexer) -> Self {
        indexer.get_or_create_id(raw)
    }

    fn into_serializable(self, indexer: &mut super::ConfigIndexer) -> anyhow::Result<Self::Raw> {
        indexer.get_label(self).map(Clone::clone)
    }
}

impl<C> Display for ConfigLabel<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
