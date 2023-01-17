use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::ConfigIndexer;

pub trait Serializable {
    type Raw: Serialize + for<'a> Deserialize<'a>;
    fn from_serializable(raw: Self::Raw, indexer: &mut ConfigIndexer) -> Self;
    fn into_serializable(self, indexer: &mut ConfigIndexer) -> anyhow::Result<Self::Raw>;
}

macro_rules! trivially_serializable {
    ($ty:ty) => {
        impl Serializable for $ty {
            type Raw = $ty;

            fn from_serializable(raw: Self::Raw, _indexer: &mut ConfigIndexer) -> Self {
                raw
            }

            fn into_serializable(self, _indexer: &mut ConfigIndexer) -> anyhow::Result<Self::Raw> {
                Ok(self)
            }
        }
    };
}

trivially_serializable!(crate::sim::units::ResourceAmount);
trivially_serializable!(crate::sim::units::ResourceWeight);
trivially_serializable!(crate::sim::units::Ticks);

impl<T: Serializable> Serializable for Vec<T> {
    type Raw = Vec<T::Raw>;

    fn from_serializable(raw: Self::Raw, indexer: &mut ConfigIndexer) -> Self {
        raw.into_iter()
            .map(|r| Serializable::from_serializable(r, indexer))
            .collect()
    }

    fn into_serializable(self, indexer: &mut ConfigIndexer) -> anyhow::Result<Self::Raw> {
        self.into_iter()
            .map(|r| Serializable::into_serializable(r, indexer))
            .try_collect()
    }
}

impl<K: Serializable + std::hash::Hash + Eq, V: Serializable> Serializable for HashMap<K, V>
where
    K::Raw: std::hash::Hash + Eq,
{
    type Raw = HashMap<K::Raw, V::Raw>;

    fn from_serializable(raw: Self::Raw, indexer: &mut ConfigIndexer) -> Self {
        raw.into_iter()
            .map(|(k, v)| {
                (
                    K::from_serializable(k, indexer),
                    V::from_serializable(v, indexer),
                )
            })
            .collect()
    }

    fn into_serializable(self, indexer: &mut ConfigIndexer) -> anyhow::Result<Self::Raw> {
        self.into_iter()
            .map(|(k, v)| Ok((k.into_serializable(indexer)?, v.into_serializable(indexer)?)))
            .try_collect()
    }
}
