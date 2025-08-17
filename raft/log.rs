use crate::{Index, Term};

pub struct LogEntry<T> {
    data: T,
    term: Term,
    index: Index,
}

pub trait Log<T>: Default {
    async fn append(&mut self, data: T);
}
