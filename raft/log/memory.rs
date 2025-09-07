use std::fmt::Debug;

use crate::log::{Log, LogEntry};

#[derive(Debug, Default)]
pub struct MemoryLog {
    data: Vec<LogEntry>,
}

impl Log for MemoryLog {
    fn len(&self) -> crate::Index {
        self.data.len() as u64
    }

    fn last_term(&self) -> Option<crate::Term> {
        self.data.last().map(|entry| entry.term)
    }

    async fn read_entry(&self, idx: crate::Index) -> super::LogResult<Option<super::LogEntry>> {
        Ok(self.data.get(idx as usize).cloned())
    }

    async fn read_entry_v(
        &self,
        range: std::ops::Range<crate::Index>,
    ) -> super::LogResult<Option<Vec<super::LogEntry>>> {
        Ok((!range.is_empty()).then(|| {
            let range = range.start as usize..range.end as usize;
            self.data[range].to_vec()
        }))
    }

    async fn truncate(&mut self, index: crate::Index) -> super::LogResult<()> {
        self.data.truncate(index as usize);
        Ok(())
    }
}
