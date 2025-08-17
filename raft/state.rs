use crate::{Index, NodeId, Term};

#[derive(Debug, Default)]
pub struct RaftState {
    pub persistent_state: PersistentState,
    /// Index of the last Log Entry that was appended
    pub last_applied_index: Index,
    pub last_term: Term,
}

#[derive(Debug, Default)]
pub struct PersistentState {
    /// Current Raft term
    pub current_term: Term,
    /// CandidateId that received vote in current term (or `None` if none)
    pub voted_for: Option<NodeId>,
    /// Index of the last Log Entry that was commited
    pub last_commit_index: Index,
}
