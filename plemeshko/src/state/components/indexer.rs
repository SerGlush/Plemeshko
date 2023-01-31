use anyhow::Result;

use crate::state::raw_indexer::RawIndexer;

use super::{ComponentId, ComponentLabel, ComponentSlotId, RawComponentId};

#[derive(Default)]
pub struct ComponentIndexer(pub(super) RawIndexer<String, RawComponentId>);

impl ComponentIndexer {
    pub fn id(&self, label: &ComponentLabel) -> Result<ComponentId> {
        self.0.id(&label.0).map(ComponentId)
    }

    pub fn label(&self, id: ComponentId) -> Result<&ComponentLabel> {
        unsafe { std::mem::transmute(self.0.label(id.0)) }
    }

    pub fn indices(&self) -> impl Iterator<Item = ComponentSlotId> {
        (0..self.0.id_to_label.len()).map(ComponentSlotId)
    }
}
