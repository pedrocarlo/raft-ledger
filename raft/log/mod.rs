use std::{fmt::Debug, ops::Range};

use bytes::Bytes;

use crate::{Index, Term};

pub mod memory;

pub type LogError = ();
pub type LogResult<T> = core::result::Result<T, LogError>;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub data: Bytes,
    pub term: Term,
}

pub trait Log {
    fn len(&self) -> Index;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn last_term(&self) -> Option<Term>;
    fn last_index(&self) -> Option<Index> {
        let len = self.len();
        if len == 0 { None } else { Some(len) }
    }
    /// Read log entry.
    ///
    /// Returns None if index is out of bounds
    fn read_entry(
        &self,
        idx: Index,
    ) -> impl std::future::Future<Output = LogResult<Option<LogEntry>>> + Send;
    /// Read contiguous log entries.
    ///
    /// # Panics
    /// Returns None if range is out of bounds
    fn read_entry_v(
        &self,
        range: Range<Index>,
    ) -> impl std::future::Future<Output = LogResult<Option<Vec<LogEntry>>>> + Send;
    /// Truncates the log up to `index`
    fn truncate(&mut self, index: Index)
    -> impl std::future::Future<Output = LogResult<()>> + Send;
}
