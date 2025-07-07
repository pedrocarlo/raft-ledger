use async_trait::async_trait;

pub type Term = u128;
pub type Index = u128;
pub type NodeId = u32;

pub enum NodeType {
    Leader,
    Candidate,
    Follower,
}

pub struct Node<T> {
    node_type: NodeType,
    /// Current Raft term
    current_term: Term,
    /// Index of the last Log Entry that was commited
    last_commit_index: Index,
    /// Index of the last Log Entry that was appended
    last_applied_index: Index,
    election_timeout: std::time::Duration,
    heartbeat_interval: std::time::Duration,
    id: NodeId,
    peers: Vec<NodeId>,
    log: Box<dyn Log<T>>,
}

pub struct LogEntry<T> {
    data: T,
    term: Term,
    index: Index,
}

#[async_trait]
pub trait Log<T> {
    async fn append(&mut self, data: T);
}
