mod app;
mod id;
mod indexer;
mod loader;
mod shared;

pub use app::*;
pub use id::*;
pub use indexer::*;
pub use loader::*;
pub use shared::*;

#[derive(Clone, Copy)]
pub struct ComponentsRef<'a> {
    pub indexer: &'a ComponentIndexer,
    pub app: &'a AppComponents,
    pub shared: &'a SharedComponents,
}

// todo: parallelize component loading?
// todo: partially loaded component? (no App/Shared part?)
