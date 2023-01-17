mod id;
mod indexer;
mod repository;
mod repository_builder;
mod serializable;

pub use id::*;
pub use indexer::*;
pub use repository::*;
pub use repository_builder::*;
pub use serializable::*;

use serde::Deserialize;

pub trait Config: Sized + Send + Sync + 'static {
    const TAG: &'static str;
    type Raw: for<'a> Deserialize<'a>;

    fn prepare(raw: Self::Raw, id: ConfigLabel<Self>, indexer: &mut ConfigIndexer) -> Self;
    // fn validate_ids(&self, indexer: &ConfigIndexer) -> ?
}
