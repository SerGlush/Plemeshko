use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use anyhow::Result;

use super::components::ComponentsRef;

pub trait Serializable: Sized {
    type Raw; //: Serialize + for<'a> Deserialize<'a>;
    fn from_serializable(raw: Self::Raw, ctx: ComponentsRef<'_>) -> Result<Self>;
    fn into_serializable(self, ctx: ComponentsRef<'_>) -> Result<Self::Raw>;
}

macro_rules! trivially_serializable {
    ($ty:ty) => {
        impl crate::state::serializable::Serializable for $ty {
            type Raw = $ty;

            fn from_serializable(
                raw: Self::Raw,
                _ctx: crate::state::components::ComponentsRef<'_>,
            ) -> anyhow::Result<Self> {
                Ok(raw)
            }

            fn into_serializable(
                self,
                _ctx: crate::state::components::ComponentsRef<'_>,
            ) -> anyhow::Result<Self::Raw> {
                Ok(self)
            }
        }
    };
}

impl<T: Serializable> Serializable for Vec<T> {
    type Raw = Vec<T::Raw>;

    fn from_serializable(raw: Self::Raw, ctx: ComponentsRef<'_>) -> Result<Self> {
        raw.into_iter()
            .map(|r| Serializable::from_serializable(r, ctx))
            .try_collect()
    }

    fn into_serializable(self, ctx: ComponentsRef<'_>) -> anyhow::Result<Self::Raw> {
        self.into_iter()
            .map(|r| Serializable::into_serializable(r, ctx))
            .try_collect()
    }
}

impl<K: Serializable + Hash + Eq> Serializable for HashSet<K>
where
    K::Raw: Hash + Eq,
{
    type Raw = HashSet<K::Raw>;

    fn from_serializable(raw: Self::Raw, ctx: ComponentsRef<'_>) -> Result<Self> {
        raw.into_iter()
            .map(|k| K::from_serializable(k, ctx))
            .try_collect()
    }

    fn into_serializable(self, ctx: ComponentsRef<'_>) -> Result<Self::Raw> {
        self.into_iter()
            .map(|k| k.into_serializable(ctx))
            .try_collect()
    }
}

impl<K: Serializable + Hash + Eq, V: Serializable> Serializable for HashMap<K, V>
where
    K::Raw: Hash + Eq,
{
    type Raw = HashMap<K::Raw, V::Raw>;

    fn from_serializable(raw: Self::Raw, ctx: ComponentsRef<'_>) -> Result<Self> {
        raw.into_iter()
            .map(|(k, v)| Ok((K::from_serializable(k, ctx)?, V::from_serializable(v, ctx)?)))
            .try_collect()
    }

    fn into_serializable(self, ctx: ComponentsRef<'_>) -> anyhow::Result<Self::Raw> {
        self.into_iter()
            .map(|(k, v)| Ok((k.into_serializable(ctx)?, v.into_serializable(ctx)?)))
            .try_collect()
    }
}
