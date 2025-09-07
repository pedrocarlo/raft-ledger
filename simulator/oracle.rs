use raft::log::LogEntry;

#[derive(Debug, Default)]
pub struct Oracle {
    data: Vec<LogEntry>,
}
