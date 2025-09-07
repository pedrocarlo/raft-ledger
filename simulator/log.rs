use raft::{
    Index, Term,
    log::{Log, LogEntry, LogResult},
};

pub struct SimLog {
    data: Vec<LogEntry>,
    last_term: Option<Term>,
}

impl Log for SimLog {
    fn len(&self) -> Index {
        self.data.len() as u64
    }

    fn last_term(&self) -> Option<Term> {
        self.last_term
    }

    async fn read_entry(&self, idx: Index) -> LogResult<LogEntry> {
        self.data
            .get(idx as usize)
            .cloned()
            .ok_or_else(|| todo!("Set a proper error here"))
    }

    async fn read_entry_v(&self, range: std::ops::Range<Index>) -> LogResult<Vec<LogEntry>> {
        assert!(!range.is_empty());
        let range = range.start as usize..range.end as usize;
        Ok(self.data[range].to_vec())
    }

    async fn truncate(&mut self, index: Index) -> LogResult<()> {
        self.data.truncate(index as usize);
        Ok(())
    }
}
