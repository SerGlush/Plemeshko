use std::{collections::HashMap, hash::Hash};

use serde::Deserialize;

use crate::state::text::TextIdFactory;

use super::ConfigsLoadingContext;

/// Trait for types which after deserialization are "prepared" and put into the final config.
/// It differs from [`Serializable`](crate::state::Serializable) by using different context and supporting only deserialization.
pub trait Prepare: for<'a> Deserialize<'a> {
    type Prepared;

    fn prepare(
        self,
        ctx: &mut ConfigsLoadingContext<'_>,
        tif: &mut TextIdFactory,
    ) -> anyhow::Result<Self::Prepared>;
}

macro_rules! trivial_config_prepare {
    ($ty:ty) => {
        impl crate::state::config::Prepare for $ty {
            type Prepared = Self;

            fn prepare(
                self,
                _ctx: &mut crate::state::config::ConfigsLoadingContext<'_>,
                _tif: &mut crate::state::text::TextIdFactory,
            ) -> anyhow::Result<Self::Prepared> {
                Ok(self)
            }
        }
    };
}

impl<T: Prepare> Prepare for Vec<T> {
    type Prepared = Vec<T::Prepared>;

    fn prepare(
        self,
        ctx: &mut ConfigsLoadingContext<'_>,
        tif: &mut TextIdFactory,
    ) -> anyhow::Result<Self::Prepared> {
        self.into_iter()
            .enumerate()
            .map(|(i, raw)| tif.with_branch(&i.to_string(), |tif| raw.prepare(ctx, tif)))
            .try_collect()
    }
}

impl<K: Hash + Eq + Prepare<Prepared: Hash + Eq>, V: Prepare> Prepare for HashMap<K, V> {
    type Prepared = HashMap<K::Prepared, V::Prepared>;

    fn prepare(
        self,
        ctx: &mut ConfigsLoadingContext<'_>,
        tif: &mut TextIdFactory,
    ) -> anyhow::Result<Self::Prepared> {
        tif.with_lock(|tif| {
            self.into_iter()
                .map(|(k, v)| Ok((k.prepare(ctx, tif)?, v.prepare(ctx, tif)?)))
                .try_collect()
        })
    }
}
