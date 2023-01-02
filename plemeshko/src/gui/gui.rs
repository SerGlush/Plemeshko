use std::sync::{Arc, Mutex};

use crate::sim::Sim;

use super::error::GuiResult;

pub struct Gui {}

impl Gui {
    pub(super) fn init(sim: &Sim) -> GuiResult<Self> {
        Ok(Gui {})
    }

    pub(super) fn update(&mut self, sim: Arc<Mutex<Sim>>) -> GuiResult<()> {
        Ok(())
    }
}
