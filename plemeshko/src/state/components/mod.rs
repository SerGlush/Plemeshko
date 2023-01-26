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

// todo: parallelize component loading?
// todo: partially loaded component? (no App/Shared part?)
