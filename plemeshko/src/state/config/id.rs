use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
    marker::PhantomData,
};

use anyhow::{anyhow, Result};
use educe::Educe;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::state::{
    components::{concat_label, split_label, ComponentId, ComponentLabel, ComponentsRef},
    serializable::Serializable,
    text::TextIdFactory,
};

use super::{prepare::Prepare, Config, ConfigIndexerMap, ConfigsLoadingContext};

#[derive(Educe, Deserialize)]
#[educe(Clone, Debug, PartialEq, Eq, Hash)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ConfigLabel<C>(pub(super) String, pub(super) PhantomData<C>);

#[derive(Educe)]
#[educe(Hash, PartialEq, Eq, Debug, Clone)]
pub struct FatConfigLabel<C>(
    /// Component prefix of the label, `None` when the label is local (allowed only inside components).
    pub Option<ComponentLabel>,
    pub ConfigLabel<C>,
);

pub(super) type RawConfigId = u32;

#[derive(Educe)]
#[educe(Hash, PartialEq, Eq, Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ConfigId<C>(pub(super) RawConfigId, PhantomData<C>);

#[derive(Educe)]
#[educe(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FatConfigId<C>(pub ComponentId, pub ConfigId<C>);

impl<C> ConfigLabel<C> {
    pub(super) fn new(label: String) -> Self {
        ConfigLabel(label, PhantomData)
    }
}

impl<C> ConfigId<C> {
    pub(super) fn new(id: RawConfigId) -> Self {
        ConfigId(id, PhantomData)
    }

    pub fn in_component(self, component_id: ComponentId) -> FatConfigId<C> {
        FatConfigId(component_id, self)
    }

    pub fn in_core(self) -> FatConfigId<C> {
        self.in_component(ComponentId::core())
    }
}

impl<C> FatConfigId<C> {
    pub fn new_core(id: ConfigId<C>) -> Self {
        FatConfigId(ComponentId::core(), id)
    }
}

impl<C> From<String> for ConfigLabel<C> {
    fn from(value: String) -> Self {
        ConfigLabel(value, PhantomData)
    }
}

impl<C> Display for ConfigLabel<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<C: Config> Prepare for ConfigLabel<C> {
    type Prepared = ConfigId<C>;

    fn prepare(
        self,
        ctx: &mut ConfigsLoadingContext<'_>,
        _tif: &mut TextIdFactory,
    ) -> anyhow::Result<ConfigId<C>> {
        ctx.st.get_or_create_id(Cow::Owned(self))
    }
}

impl<'de, C: Config> Deserialize<'de> for FatConfigLabel<C> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let (comp, postfix) = split_label::<D>(&raw)?;
        Ok(FatConfigLabel(comp, ConfigLabel::new(postfix.to_owned())))
    }
}

impl<C: Config> Serialize for FatConfigLabel<C> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let raw = concat_label(self.0.clone(), &self.1 .0);
        serializer.serialize_str(&raw)
    }
}

impl<C: Config> Prepare for FatConfigLabel<C> {
    type Prepared = FatConfigId<C>;

    fn prepare(
        self,
        ctx: &mut ConfigsLoadingContext<'_>,
        _tif: &mut TextIdFactory,
    ) -> Result<FatConfigId<C>> {
        Ok(match self.0 {
            Some(comp_id) => {
                let comp_id = ctx.components.indexer.get_id(&comp_id)?;
                let cfg_id = ctx
                    .components
                    .shared
                    .get_component(comp_id)?
                    .configs
                    .get_indexer::<C>()?
                    .get_id(&self.1)?;
                FatConfigId(comp_id, cfg_id)
            }
            None => {
                let cfg_id = ctx.st.get_or_create_id(Cow::Owned(self.1))?;
                FatConfigId(ctx.component_id(), cfg_id)
            }
        })
    }
}

impl<C: Config> Serializable for FatConfigId<C> {
    type Raw = FatConfigLabel<C>;

    fn from_serializable(raw: Self::Raw, ctx: &ComponentsRef<'_>) -> Result<Self> {
        let comp_label = &raw.0.ok_or_else(|| {
            anyhow!(
                "Deserializing local label outside any component: ?/{}",
                raw.1
            )
        })?;
        let comp_id = ctx.indexer.get_id(comp_label)?;
        let cfg_id = ctx
            .shared
            .get_component(comp_id)?
            .configs
            .get_indexer::<C>()?
            .get_id(&raw.1)?;
        Ok(FatConfigId(comp_id, cfg_id))
    }

    fn into_serializable(self, ctx: &ComponentsRef<'_>) -> Result<Self::Raw> {
        let comp_label = ctx.indexer.get_label(self.0)?;
        let cfg_label = ctx
            .shared
            .get_component(self.0)?
            .configs
            .get_label(self.1)?;
        Ok(FatConfigLabel(Some(comp_label.clone()), cfg_label.clone()))
    }
}
