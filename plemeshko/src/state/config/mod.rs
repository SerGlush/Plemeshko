mod id;
mod indexer;
mod repository;
mod repository_builder;
mod type_registry;
#[macro_use]
mod prepare;

pub use id::*;
pub use indexer::*;
pub use prepare::*;
pub use repository::*;
pub use repository_builder::*;
pub use type_registry::*;

use anyhow::Result;

use super::{
    components::{ComponentIndexer, SharedComponents},
    text::TextIdFactory,
};

pub type ConfigFinalizationPriority = i32;

/// A trait for small assets with custom format or schema, unlike, for example, textures or sounds.
/// Loaded as a part of a component, initially as [`Raw`](Config::Raw) and then "prepared".
/// [`TAG`](Config::TAG) is used to differentiate between different [`Config`](Config) types.
pub trait Config: Sized + Send + Sync + 'static {
    type Raw: Prepare<Prepared = Self>;

    const TAG: &'static str;
    const FINALIZATION_PRIORITY: ConfigFinalizationPriority = 0;

    #[allow(unused_variables)]
    fn finalize(indexer: &ComponentIndexer, shared_comps: &mut SharedComponents) -> Result<()> {
        Ok(())
    }
}

pub fn create_config_text_id_factory<C: Config>(config_label: &ConfigLabel<C>) -> TextIdFactory {
    TextIdFactory::with_capacity(C::TAG.len() + config_label.0.len() + 2)
        .branch(C::TAG)
        .branch(&config_label.0)
}
