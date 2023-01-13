pub trait Memento {
    type T;
    fn state(&self) -> Self::T;
    fn restore(&mut self, state: Self::T);
}
