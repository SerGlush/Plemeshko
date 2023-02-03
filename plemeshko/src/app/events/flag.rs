use std::cell::Cell;

pub struct FlagEvent(Cell<bool>);

impl FlagEvent {
    pub fn new() -> Self {
        FlagEvent(Cell::new(false))
    }

    pub fn emit(&self) {
        assert!(!self.emit_idem());
    }

    /// Returns `true` when the flag was already set.
    pub fn emit_idem(&self) -> bool {
        self.0.replace(true)
    }

    pub fn get(&self) -> bool {
        self.0.get()
    }
}
