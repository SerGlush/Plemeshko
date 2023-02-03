use std::cell::Cell;

pub struct SetEvent<T>(Cell<Option<T>>);

impl<T> SetEvent<T> {
    pub fn new() -> Self {
        SetEvent(Cell::new(None))
    }

    pub fn emit(&self, value: T) {
        assert!(self.emit_idem(value).is_none());
    }

    /// Returns old value if it was set.
    pub fn emit_idem(&self, value: T) -> Option<T> {
        self.0.replace(Some(value))
    }

    pub fn get(&self) -> Option<T>
    where
        T: Copy,
    {
        self.0.get()
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.0.get_mut().as_mut()
    }
}
