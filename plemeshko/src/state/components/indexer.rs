use anyhow::Result;

use crate::state::indexer::Indexer;

use super::{ComponentId, ComponentLabel, RawComponentId};

#[derive(Default)]
pub struct ComponentIndexer(pub(super) Indexer<String, RawComponentId>);

impl ComponentIndexer {
    pub fn get_id(&self, label: &ComponentLabel) -> Result<ComponentId> {
        self.0.get_id(&label.0).map(ComponentId)
    }

    pub fn get_label(&self, id: ComponentId) -> Result<&ComponentLabel> {
        unsafe { std::mem::transmute(self.0.get_label(id.0)) }
    }
}
