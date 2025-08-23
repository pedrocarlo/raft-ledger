use bytes::Bytes;

use crate::{Index, Term};

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub data: Bytes,
    pub term: Term,
}

pub trait Log: Default {
    async fn append(&mut self, entry: LogEntry);
    fn len(&self) -> u64;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn last_term(&self) -> Option<Term>;
    fn last_index(&self) -> Option<Index> {
        let len = self.len();
        if len == 0 { None } else { Some(len) }
    }
    /// Get's the last Log entry
    async fn last(&self) -> LogEntry;
}
