use crate::{Index, NodeId, Term};

#[derive(Debug, Default)]
pub struct RaftState {
    pub persistent_state: PersistentState,
    /// Index of the last Log Entry that was appended
    pub last_applied_index: Index,
    pub last_term: Term,
}

impl RaftState {}

#[derive(Debug, Default)]
pub struct PersistentState {
    /// Current Raft term
    current_term: Term,
    /// CandidateId that received vote in current term (or `None` if none)
    voted_for: Option<NodeId>,
    /// Index of the last Log Entry that was commited
    last_commit_index: Index,
    /// Tracks is persistent state was changed
    state_dirty: bool,
}

impl PersistentState {
    pub fn current_term(&self) -> Term {
        self.current_term
    }

    pub fn set_current_term(&mut self, term: Term) {
        self.state_dirty = true;
        self.current_term = term;
    }

    pub fn voted_for(&self) -> Option<NodeId> {
        self.voted_for
    }

    pub fn set_voted_for(&mut self, voted_for: Option<NodeId>) {
        self.state_dirty = true;
        self.voted_for = voted_for;
    }

    pub fn last_commit_index(&self) -> Index {
        self.last_commit_index
    }

    pub fn set_last_commit_index(&mut self, index: Index) {
        self.state_dirty = true;
        self.last_commit_index = index;
    }
}
