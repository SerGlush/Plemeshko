use std::collections::HashMap;

use anyhow::Result;

use crate::state::components::SharedComponents;

use super::components::{AppComponents, ComponentIndexer};

pub struct SerializationContext<'a> {
    pub component_indexer: &'a ComponentIndexer,
    pub app_components: &'a AppComponents,
    pub shared_components: &'a mut SharedComponents,
}

pub trait Serializable: Sized {
    type Raw; //: Serialize + for<'a> Deserialize<'a>;
    fn from_serializable(raw: Self::Raw, ctx: &mut SerializationContext<'_>) -> Result<Self>;
    fn into_serializable(self, ctx: &SerializationContext<'_>) -> Result<Self::Raw>;
}

macro_rules! trivially_serializable {
    ($ty:ty) => {
        impl crate::state::serializable::Serializable for $ty {
            type Raw = $ty;

            fn from_serializable(
                raw: Self::Raw,
                _ctx: &mut crate::state::serializable::SerializationContext<'_>,
            ) -> anyhow::Result<Self> {
                Ok(raw)
            }

            fn into_serializable(
                self,
                _ctx: &crate::state::serializable::SerializationContext<'_>,
            ) -> anyhow::Result<Self::Raw> {
                Ok(self)
            }
        }
    };
}

impl<T: Serializable> Serializable for Vec<T> {
    type Raw = Vec<T::Raw>;

    fn from_serializable(raw: Self::Raw, ctx: &mut SerializationContext<'_>) -> Result<Self> {
        raw.into_iter()
            .map(|r| Serializable::from_serializable(r, ctx))
            .try_collect()
    }

    fn into_serializable(self, ctx: &SerializationContext<'_>) -> anyhow::Result<Self::Raw> {
        self.into_iter()
            .map(|r| Serializable::into_serializable(r, ctx))
            .try_collect()
    }
}

impl<K: Serializable + std::hash::Hash + Eq, V: Serializable> Serializable for HashMap<K, V>
where
    K::Raw: std::hash::Hash + Eq,
{
    type Raw = HashMap<K::Raw, V::Raw>;

    fn from_serializable(raw: Self::Raw, ctx: &mut SerializationContext<'_>) -> Result<Self> {
        raw.into_iter()
            .map(|(k, v)| Ok((K::from_serializable(k, ctx)?, V::from_serializable(v, ctx)?)))
            .try_collect()
    }

    fn into_serializable(self, ctx: &SerializationContext<'_>) -> anyhow::Result<Self::Raw> {
        self.into_iter()
            .map(|(k, v)| Ok((k.into_serializable(ctx)?, v.into_serializable(ctx)?)))
            .try_collect()
    }
}
