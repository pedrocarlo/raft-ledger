use crate::{Index, Term};

#[derive(Debug, Clone)]
pub struct LogEntry<E> {
    pub data: E,
    pub meta: LogEntryMeta,
}

#[derive(Debug, Clone, Copy)]
pub struct LogEntryMeta {
    pub term: Term,
    pub index: Index,
}

pub trait Log<E>: Default {
    async fn append(&mut self, entry: LogEntry<E>);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Get's the last Log entry
    async fn last(&self) -> LogEntry<E>;
    /// Get's the last Log entry term and index
    ///
    /// Separate function to allow an optimized get of the last entry term
    /// wihtout deserializing the whole entry
    fn last_entry_meta(&self) -> LogEntryMeta;
}

pub struct MemoryLog {}
