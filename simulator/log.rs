use raft::{
    Index, Term,
    log::{Log, LogEntry, LogResult},
};

#[derive(Debug, Default)]
pub struct SimLog {
    data: Vec<LogEntry>,
}

impl Log for SimLog {
    fn len(&self) -> Index {
        self.data.len() as u64
    }

    fn last_term(&self) -> Option<Term> {
        self.data.last().map(|entry| entry.term)
    }

    async fn read_entry(&self, idx: Index) -> LogResult<Option<LogEntry>> {
        Ok(self.data.get(idx as usize).cloned())
    }

    async fn read_entry_v(
        &self,
        range: std::ops::Range<Index>,
    ) -> LogResult<Option<Vec<LogEntry>>> {
        Ok((!range.is_empty()).then(|| {
            let range = range.start as usize..range.end as usize;
            self.data[range].to_vec()
        }))
    }

    async fn truncate(&mut self, index: Index) -> LogResult<()> {
        self.data.truncate(index as usize);
        Ok(())
    }
}
