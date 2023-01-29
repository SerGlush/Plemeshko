use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
    marker::PhantomData,
};

use anyhow::Result;
use educe::Educe;
use serde::{Deserialize, Serialize};

use crate::state::{
    components::{ComponentId, ComponentsRef, RawFatLabel},
    serializable::Serializable,
    text::TextIdFactory,
};

use super::{prepare::Prepare, Config, ConfigIndexerMap, ConfigsLoadingContext};

#[derive(Educe, Deserialize)]
#[educe(Clone, Debug, PartialEq, Eq, Hash)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ConfigLabel<C>(pub(super) String, pub(super) PhantomData<C>);

#[derive(Educe, Serialize, Deserialize)]
#[educe(Hash, PartialEq, Eq, Debug, Clone)]
#[serde(transparent)]
#[repr(transparent)]
pub struct FatConfigLabel<C>(RawFatLabel, PhantomData<C>);

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

impl<C> FatConfigLabel<C> {
    pub fn config(&self) -> &ConfigLabel<C> {
        unsafe { std::mem::transmute(&self.0 .1) }
    }

    pub fn into_config(self) -> ConfigLabel<C> {
        ConfigLabel::new(self.0 .1)
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

impl<C: Config> Prepare for FatConfigLabel<C> {
    type Prepared = FatConfigId<C>;

    fn prepare(
        self,
        ctx: &mut ConfigsLoadingContext<'_>,
        _tif: &mut TextIdFactory,
    ) -> Result<FatConfigId<C>> {
        Ok(match &self.0 .0 {
            Some(comp_label) => {
                let comp_id = ctx.other_components.indexer.get_id(comp_label)?;
                let cfg_id = ctx
                    .other_components
                    .shared
                    .get_component(comp_id)?
                    .configs
                    .get_indexer::<C>()?
                    .get_id(self.config())?;
                FatConfigId(comp_id, cfg_id)
            }
            None => {
                let cfg_id = ctx.st.get_or_create_id(Cow::Owned(self.into_config()))?;
                FatConfigId(ctx.this_component.id(), cfg_id)
            }
        })
    }
}

impl<C: Config> Serializable for FatConfigId<C> {
    type Raw = FatConfigLabel<C>;

    fn from_serializable(raw: Self::Raw, comps: ComponentsRef<'_>) -> Result<Self> {
        let comp_id = raw.0.deserialize_component_id(comps)?;
        let cfg_id = comps
            .shared
            .get_component(comp_id)?
            .configs
            .get_indexer::<C>()?
            .get_id(raw.config())?;
        Ok(FatConfigId(comp_id, cfg_id))
    }

    fn into_serializable(self, comps: ComponentsRef<'_>) -> Result<Self::Raw> {
        let comp_label = comps.indexer.get_label(self.0)?;
        let cfg_label = comps
            .shared
            .get_component(self.0)?
            .configs
            .get_label(self.1)?;
        Ok(FatConfigLabel(
            RawFatLabel(Some(comp_label.clone()), cfg_label.0.clone()),
            PhantomData,
        ))
    }
}
