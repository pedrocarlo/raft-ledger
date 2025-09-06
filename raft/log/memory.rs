use std::fmt::Debug;

use crate::{
    Term,
    log::{Log, LogEntry},
};

#[derive(Debug, Default)]
pub struct MemoryLog {
    data: Vec<LogEntry>,
    last_term: Option<Term>,
}

impl Log for MemoryLog {
    fn len(&self) -> crate::Index {
        self.data.len() as u64
    }

    fn last_term(&self) -> Option<crate::Term> {
        self.last_term
    }

    async fn read_entry(&self, idx: crate::Index) -> super::LogResult<super::LogEntry> {
        self.data
            .get(idx as usize)
            .cloned()
            .ok_or_else(|| todo!("Set a proper error here"))
    }

    async fn read_entry_v(
        &self,
        range: std::ops::Range<crate::Index>,
    ) -> super::LogResult<Vec<super::LogEntry>> {
        assert!(!range.is_empty());
        let range = range.start as usize..range.end as usize;
        Ok(self.data[range].to_vec())
    }

    async fn truncate(&mut self, index: crate::Index) -> super::LogResult<()> {
        self.data.truncate(index as usize);
        Ok(())
    }
}
